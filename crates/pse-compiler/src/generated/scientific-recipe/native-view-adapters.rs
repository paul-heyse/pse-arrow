use pse_contract_proto::support::{
    NativeFrom, BoundaryProjectionError, native_from, decode_view_root,
    unknown_fields_view, layout, child_path, required, refused, from_items, unique_pairs,
};
use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityClaimView<
        'buffer,
    >,
> for pse_model::applicability::Claim {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityClaimView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                .as_ref()
                .map(|value| -> Result<_, AdapterError> {
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        value,
                        &child_path(path, "id"),
                    )
                })
                .transpose()?,
            wire
                .coverage
                .as_ref()
                .map(|value| -> Result<_, AdapterError> {
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        value,
                        &child_path(path, "coverage"),
                    )
                })
                .transpose()?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.owner,
                &child_path(path, "owner"),
            )?,
            {
                let current_path = &child_path(path, "owner_lineage");
                let values = wire
                    .owner_lineage
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .evidence
                .as_ref()
                .map(|value| -> Result<_, AdapterError> {
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        value,
                        &child_path(path, "evidence"),
                    )
                })
                .transpose()?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.form,
                &child_path(path, "form"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.call,
                &child_path(path, "call"),
            )?,
            {
                let current_path = &child_path(path, "records");
                let values = wire
                    .records
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "dependencies");
                let values = wire
                    .dependencies
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValidityLayerView<
                    '_,
                >,
            >(
                required(wire.layer.as_option(), &child_path(path, "layer"))?,
                &child_path(path, "layer"),
            )?,
            wire
                .basis
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingApplicabilityBasisView<
                            '_,
                        >,
                    >(value, &child_path(path, "basis"))
                })
                .transpose()?,
            wire
                .reason
                .as_ref()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, &str>(value, &child_path(path, "reason"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityNodeView<
        'buffer,
    >,
> for pse_model::applicability::Node {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityNodeView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityClaimView<
                    '_,
                >,
            >(
                required(wire.claim.as_option(), &child_path(path, "claim"))?,
                &child_path(path, "claim"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityRegionView<
                    '_,
                >,
            >(
                required(wire.region.as_option(), &child_path(path, "region"))?,
                &child_path(path, "region"),
            )?,
            {
                let current_path = &child_path(path, "dependencies");
                let values = wire
                    .dependencies
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityNodeView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple3OfStringAndUsizeAndSemanticIdView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "permissions");
                let values = wire
                    .permissions
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityPermissionView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityPermissionView<
        'buffer,
    >,
> for pse_model::applicability::Permission {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityPermissionView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.id,
                &child_path(path, "id"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.scope,
                &child_path(path, "scope"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPermissionTargetView<
                    '_,
                >,
            >(
                required(
                    wire.target_kind.as_option(),
                    &child_path(path, "target_kind"),
                )?,
                &child_path(path, "target_kind"),
            )?,
            {
                let current_path = &child_path(path, "targets");
                let values = wire
                    .targets
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
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
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityRegionView<
        'buffer,
    >,
> for pse_model::applicability::Region {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityRegionView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::applicability_region::Kind::Predicate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Predicate");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::applicability_region::Kind::Unrestricted(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unrestricted");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::applicability_region::Kind::Unknown(
                wire,
            ) => {
                let alternative_path = child_path(path, "Unknown");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::applicability_region::Kind::Union(
                wire,
            ) => {
                let alternative_path = child_path(path, "Union");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityNodeView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
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
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Array2ModelingResolvedNodeIxView<
        'buffer,
    >,
> for [pse_modeling::resolved::NodeIx; 2] {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Array2ModelingResolvedNodeIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <[pse_modeling::resolved::NodeIx; 2] as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <[pse_modeling::resolved::NodeIx; 2] as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "values")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "values");
                let values = wire
                    .values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Array8PhysicalRatioView<
        'buffer,
    >,
> for [pse_quantity::dimension::Ratio; 8] {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Array8PhysicalRatioView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredBinaryOperatorView<
        'buffer,
    >,
> for pse_authoring::dsl::BinaryOp {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredBinaryOperatorView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_binary_operator::Kind::Add(
                wire,
            ) => {
                let alternative_path = child_path(path, "Add");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_binary_operator::Kind::Sub(
                wire,
            ) => {
                let alternative_path = child_path(path, "Sub");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_binary_operator::Kind::Mul(
                wire,
            ) => {
                let alternative_path = child_path(path, "Mul");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_binary_operator::Kind::Div(
                wire,
            ) => {
                let alternative_path = child_path(path, "Div");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_binary_operator::Kind::Pow(
                wire,
            ) => {
                let alternative_path = child_path(path, "Pow");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::BinaryOp as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredBuiltinFunctionView<
        'buffer,
    >,
> for pse_quantity::functions::Function {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredBuiltinFunctionView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Exp(
                wire,
            ) => {
                let alternative_path = child_path(path, "Exp");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Log(
                wire,
            ) => {
                let alternative_path = child_path(path, "Log");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Sqrt(
                wire,
            ) => {
                let alternative_path = child_path(path, "Sqrt");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Abs(
                wire,
            ) => {
                let alternative_path = child_path(path, "Abs");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Sin(
                wire,
            ) => {
                let alternative_path = child_path(path, "Sin");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Cos(
                wire,
            ) => {
                let alternative_path = child_path(path, "Cos");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Min(
                wire,
            ) => {
                let alternative_path = child_path(path, "Min");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Max(
                wire,
            ) => {
                let alternative_path = child_path(path, "Max");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Convert(
                wire,
            ) => {
                let alternative_path = child_path(path, "Convert");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_builtin_function::Kind::Broadcast(
                wire,
            ) => {
                let alternative_path = child_path(path, "Broadcast");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_quantity::functions::Function as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredComparisonOperatorView<
        'buffer,
    >,
> for pse_authoring::dsl::CompareOp {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredComparisonOperatorView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_comparison_operator::Kind::Eq(
                wire,
            ) => {
                let alternative_path = child_path(path, "Eq");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_comparison_operator::Kind::NotEq(
                wire,
            ) => {
                let alternative_path = child_path(path, "NotEq");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_comparison_operator::Kind::Lt(
                wire,
            ) => {
                let alternative_path = child_path(path, "Lt");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_comparison_operator::Kind::Le(
                wire,
            ) => {
                let alternative_path = child_path(path, "Le");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_comparison_operator::Kind::Gt(
                wire,
            ) => {
                let alternative_path = child_path(path, "Gt");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_comparison_operator::Kind::Ge(
                wire,
            ) => {
                let alternative_path = child_path(path, "Ge");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::CompareOp as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredReductionKindView<
        'buffer,
    >,
> for pse_authoring::dsl::ReduceKind {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredReductionKindView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_reduction_kind::Kind::Sum(
                wire,
            ) => {
                let alternative_path = child_path(path, "Sum");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::ReduceKind as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_authoring::dsl::ReduceKind as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::ReduceKind as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_reduction_kind::Kind::Prod(
                wire,
            ) => {
                let alternative_path = child_path(path, "Prod");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::ReduceKind as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_authoring::dsl::ReduceKind as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::ReduceKind as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::authored_reduction_kind::Kind::Integral(
                wire,
            ) => {
                let alternative_path = child_path(path, "Integral");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_authoring::dsl::ReduceKind as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_authoring::dsl::ReduceKind as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_authoring::dsl::ReduceKind as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredScientificNumberView<
        'buffer,
    >,
> for pse_authoring::dsl::Number {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredScientificNumberView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_authoring::dsl::Number as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_authoring::dsl::Number as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value"), (2, "exact_integer"), (3, "unit")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
            wire
                .exact_integer
                .as_ref()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, &[u8]>(value, &child_path(path, "exact_integer"))
                })
                .transpose()?,
            wire
                .unit
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalUnitProductView<
                            '_,
                        >,
                    >(value, &child_path(path, "unit"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BasisIdView<
        'buffer,
    >,
> for pse_quantity::ids::BasisId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BasisIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::BasisId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::BasisId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BoundIndexIdView<
        'buffer,
    >,
> for pse_quantity::ids::BoundIndexId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BoundIndexIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::BoundIndexId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::BoundIndexId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BoundIndexReferenceView<
        'buffer,
    >,
> for pse_quantity::index::BoundIndexRef {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BoundIndexReferenceView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::index::BoundIndexRef as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::index::BoundIndexRef as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "bound_index"), (2, "domain"), (3, "kind")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BoundIndexIdView<
                    '_,
                >,
            >(
                required(
                    wire.bound_index.as_option(),
                    &child_path(path, "bound_index"),
                )?,
                &child_path(path, "bound_index"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DomainIdView<
                    '_,
                >,
            >(
                required(wire.domain.as_option(), &child_path(path, "domain"))?,
                &child_path(path, "domain"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::EntityKindIdView<
                    '_,
                >,
            >(
                required(wire.kind.as_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::CompilerDirectProviderCallRecordView<
        'buffer,
    >,
> for crate::typed_math::providers::DirectProviderCallRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::CompilerDirectProviderCallRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::typed_math::providers::DirectProviderCallRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::typed_math::providers::DirectProviderCallRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "syntax"), (2, "ordinal"), (3, "prepared")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.syntax,
                &child_path(path, "syntax"),
            )?,
            native_from::<_, u64>(wire.ordinal, &child_path(path, "ordinal"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalPreparedProviderCallRecordView<
                    '_,
                >,
            >(
                required(wire.prepared.as_option(), &child_path(path, "prepared"))?,
                &child_path(path, "prepared"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::CompilerSelectedProviderCallRecordView<
        'buffer,
    >,
> for crate::typed_math::providers::SelectedProviderCallRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::CompilerSelectedProviderCallRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::typed_math::providers::SelectedProviderCallRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::typed_math::providers::SelectedProviderCallRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "function"),
                (2, "wrt"),
                (3, "descriptor"),
                (4, "output"),
                (5, "inputs"),
                (6, "result"),
                (7, "prepared"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.function,
                &child_path(path, "function"),
            )?,
            {
                let current_path = &child_path(path, "wrt");
                let values = wire
                    .wrt
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(*(value), &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.descriptor,
                &child_path(path, "descriptor"),
            )?,
            native_from::<_, u64>(wire.output, &child_path(path, "output"))?,
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
                    '_,
                >,
            >(
                required(wire.result.as_option(), &child_path(path, "result"))?,
                &child_path(path, "result"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalPreparedProviderCallRecordView<
                    '_,
                >,
            >(
                required(wire.prepared.as_option(), &child_path(path, "prepared"))?,
                &child_path(path, "prepared"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::CompilerSelectedProvidersRecordView<
        'buffer,
    >,
> for crate::typed_math::providers::SelectedProvidersRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::CompilerSelectedProvidersRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::typed_math::providers::SelectedProvidersRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::typed_math::providers::SelectedProvidersRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "identity"),
                (2, "descriptors"),
                (3, "calls"),
                (4, "direct_calls"),
                (5, "premises"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.identity,
                &child_path(path, "identity"),
            )?,
            {
                let current_path = &child_path(path, "descriptors");
                let values = wire
                    .descriptors
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfContentHashAndKernelProviderRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "calls");
                let values = wire
                    .calls
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::CompilerSelectedProviderCallRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "direct_calls");
                let values = wire
                    .direct_calls
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::CompilerDirectProviderCallRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalEvidenceRecordView<
                    '_,
                >,
            >(
                required(wire.premises.as_option(), &child_path(path, "premises"))?,
                &child_path(path, "premises"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ConversionIdView<
        'buffer,
    >,
> for pse_quantity::ids::ConversionId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ConversionIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::ConversionId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::ConversionId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
        'buffer,
    >,
> for pse_model::generated::identities::DeclarationId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::generated::identities::DeclarationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::DeclarationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DomainIdView<
        'buffer,
    >,
> for pse_quantity::ids::DomainId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DomainIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::DomainId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::DomainId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::EntityKindIdView<
        'buffer,
    >,
> for pse_quantity::ids::EntityKindId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::EntityKindIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::EntityKindId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::EntityKindId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ExternalDerivativeSourceView<
        'buffer,
    >,
> for pse_model::generated::enums::ExternalDerivativeSource {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ExternalDerivativeSourceView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::InstanceIdView<
        'buffer,
    >,
> for pse_model::generated::identities::InstanceId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::InstanceIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::generated::identities::InstanceId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::InstanceId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelDerivativeOrderView<
        'buffer,
    >,
> for pse_kernels::DerivativeOrder {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelDerivativeOrderView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::kernel_derivative_order::Kind::Value(
                wire,
            ) => {
                let alternative_path = child_path(path, "Value");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::kernel_derivative_order::Kind::First(
                wire,
            ) => {
                let alternative_path = child_path(path, "First");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::kernel_derivative_order::Kind::Second(
                wire,
            ) => {
                let alternative_path = child_path(path, "Second");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderPortView<
        'buffer,
    >,
> for pse_kernels::Port {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderPortView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_kernels::Port as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::Port as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "quantity"), (3, "unit")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                    '_,
                >,
            >(
                required(wire.quantity.as_option(), &child_path(path, "quantity"))?,
                &child_path(path, "quantity"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitIdView<
                    '_,
                >,
            >(
                required(wire.unit.as_option(), &child_path(path, "unit"))?,
                &child_path(path, "unit"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderRecordView<
        'buffer,
    >,
> for pse_kernels::portable::ProviderRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_kernels::portable::ProviderRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::portable::ProviderRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            2,
            &[(1, "version"), (2, "key"), (3, "spec")],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.key.as_ref(), &child_path(path, "key"))?,
                &child_path(path, "key"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderSpecView<
                    '_,
                >,
            >(
                required(wire.spec.as_option(), &child_path(path, "spec"))?,
                &child_path(path, "spec"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderShapeView<
        'buffer,
    >,
> for pse_kernels::ProviderShape {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderShapeView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_kernels::ProviderShape as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_kernels::ProviderShape as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "axes"), (3, "coordinates"), (4, "cells")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            {
                let current_path = &child_path(path, "axes");
                let values = wire
                    .axes
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "coordinates");
                let values = wire
                    .coordinates
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        {
                            let wrapped = value;
                            unknown_fields_view(
                                &wrapped.__buffa_unknown_fields,
                                &format!("{current_path}[{index}]"),
                            )?;
                            {
                                let current_path = &format!("{current_path}[{index}]");
                                let values = wrapped
                                    .value
                                    .iter()
                                    .enumerate()
                                    .map(|(index, value)| -> Result<_, AdapterError> {
                                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                                            value,
                                            &format!("{current_path}[{index}]"),
                                        )
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
                let current_path = &child_path(path, "cells");
                let values = wire
                    .cells
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderShapesView<
        'buffer,
    >,
> for pse_kernels::ProviderShapes {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderShapesView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderShapeView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "outputs");
                let values = wire
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderShapeView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderSpecView<
        'buffer,
    >,
> for pse_kernels::ProviderSpec {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderSpecView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderShapesView<
                    '_,
                >,
            >(
                required(wire.shapes.as_option(), &child_path(path, "shapes"))?,
                &child_path(path, "shapes"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ExternalDerivativeSourceView<
                    '_,
                >,
            >(
                required(
                    wire.derivative_source.as_option(),
                    &child_path(path, "derivative_source"),
                )?,
                &child_path(path, "derivative_source"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.revision.as_ref(), &child_path(path, "revision"))?,
                &child_path(path, "revision"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.data.as_ref(), &child_path(path, "data"))?,
                &child_path(path, "data"),
            )?,
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderPortView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "outputs");
                let values = wire
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderPortView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelDerivativeOrderView<
                    '_,
                >,
            >(
                required(
                    wire.derivatives.as_option(),
                    &child_path(path, "derivatives"),
                )?,
                &child_path(path, "derivatives"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelDerivativeOrderView<
                    '_,
                >,
            >(
                required(wire.smoothness.as_option(), &child_path(path, "smoothness"))?,
                &child_path(path, "smoothness"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KindFactorView<
        'buffer,
    >,
> for pse_quantity::kind::KindFactor {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KindFactorView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityKindIdView<
                    '_,
                >,
            >(
                required(wire.kind.as_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
                    '_,
                >,
            >(
                required(wire.exponent.as_option(), &child_path(path, "exponent"))?,
                &child_path(path, "exponent"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathBodySpecView<
        'buffer,
    >,
> for pse_math::binding::BodySpec {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathBodySpecView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.definition,
                &child_path(path, "definition"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.structure,
                &child_path(path, "structure"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.physical,
                &child_path(path, "physical"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.admissions,
                &child_path(path, "admissions"),
            )?,
            {
                let current_path = &child_path(path, "providers");
                let values = wire
                    .providers
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.policy,
                &child_path(path, "policy"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalPreparedProviderCallRecordView<
        'buffer,
    >,
> for pse_math::typed::PreparedProviderCallRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalPreparedProviderCallRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::typed::PreparedProviderCallRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::typed::PreparedProviderCallRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "identity"),
                (2, "registration"),
                (3, "partial"),
                (4, "inputs"),
                (5, "input_conversions"),
                (6, "outputs"),
                (7, "output_conversions"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.identity,
                &child_path(path, "identity"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderRecordView<
                    '_,
                >,
            >(
                required(
                    wire.registration.as_option(),
                    &child_path(path, "registration"),
                )?,
                &child_path(path, "registration"),
            )?,
            {
                let current_path = &child_path(path, "partial");
                let values = wire
                    .partial
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(*(value), &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "input_conversions");
                let values = wire
                    .input_conversions
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRepresentationConversionRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "outputs");
                let values = wire
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "output_conversions");
                let values = wire
                    .output_conversions
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRepresentationConversionRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalProofReceiptView<
        'buffer,
    >,
> for pse_math::typed::receipts::Proof {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalProofReceiptView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::typed::receipts::Proof as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::typed::receipts::Proof as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "request"), (2, "branches")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.request,
                &child_path(path, "request"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.branches,
                &child_path(path, "branches"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalProofStreamView<
        'buffer,
    >,
> for pse_math::typed::receipts::ProofStream {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalProofStreamView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::typed::receipts::ProofStream as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::typed::receipts::ProofStream as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "version"), (2, "entries")],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            {
                let current_path = &child_path(path, "entries");
                let values = wire
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalProofReceiptView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingAnnotationComponentView<
        'buffer,
    >,
> for pse_modeling::derived::AnnotationComponent {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingAnnotationComponentView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::Value(
                wire,
            ) => {
                let alternative_path = child_path(path, "Value");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::Lower(
                wire,
            ) => {
                let alternative_path = child_path(path, "Lower");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::Upper(
                wire,
            ) => {
                let alternative_path = child_path(path, "Upper");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::ObjectiveLower(
                wire,
            ) => {
                let alternative_path = child_path(path, "ObjectiveLower");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "strict")],
                    path,
                )?;
                let fields = (
                    native_from::<_, bool>(wire.strict, &child_path(path, "strict"))?,
                );
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::ObjectiveUpper(
                wire,
            ) => {
                let alternative_path = child_path(path, "ObjectiveUpper");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "strict")],
                    path,
                )?;
                let fields = (
                    native_from::<_, bool>(wire.strict, &child_path(path, "strict"))?,
                );
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::Time(
                wire,
            ) => {
                let alternative_path = child_path(path, "Time");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::Resolution(
                wire,
            ) => {
                let alternative_path = child_path(path, "Resolution");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::CriterionLower(
                wire,
            ) => {
                let alternative_path = child_path(path, "CriterionLower");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::CriterionUpper(
                wire,
            ) => {
                let alternative_path = child_path(path, "CriterionUpper");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::CaloricResponse(
                wire,
            ) => {
                let alternative_path = child_path(path, "CaloricResponse");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::CaloricThermalResolution(
                wire,
            ) => {
                let alternative_path = child_path(path, "CaloricThermalResolution");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::CaloricPowerAllowance(
                wire,
            ) => {
                let alternative_path = child_path(path, "CaloricPowerAllowance");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_annotation_component::Kind::CaloricExposure(
                wire,
            ) => {
                let alternative_path = child_path(path, "CaloricExposure");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::VERSION,
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::AnnotationComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingAnnotationKindView<
        'buffer,
    >,
> for pse_model::generated::enums::ModelingAnnotationKind {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingAnnotationKindView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingApplicabilityBasisView<
        'buffer,
    >,
> for pse_model::generated::enums::ModelingApplicabilityBasis {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingApplicabilityBasisView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingBodyInputRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::support_records::BodyInputRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingBodyInputRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::support_records::BodyInputRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::support_records::BodyInputRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "relationship"), (2, "boundary")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingDerivedRelationshipView<
                    '_,
                >,
            >(
                required(
                    wire.relationship.as_option(),
                    &child_path(path, "relationship"),
                )?,
                &child_path(path, "relationship"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryRecordView<
                    '_,
                >,
            >(
                required(wire.boundary.as_option(), &child_path(path, "boundary"))?,
                &child_path(path, "boundary"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingBoundaryFactsView<
        'buffer,
    >,
> for pse_modeling::portable::BoundaryWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingBoundaryFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_boundary_facts::Kind::Declared(
                wire,
            ) => {
                let alternative_path = child_path(path, "Declared");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::BoundaryWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::portable::BoundaryWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::BoundaryWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_boundary_facts::Kind::Bound(
                wire,
            ) => {
                let alternative_path = child_path(path, "Bound");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::BoundaryWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::portable::BoundaryWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "instance"), (2, "declaration"), (3, "coordinates")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::InstanceIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.instance.as_option(),
                            &child_path(path, "instance"),
                        )?,
                        &child_path(path, "instance"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                    {
                        let current_path = &child_path(path, "coordinates");
                        let values = wire
                            .coordinates
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingStaticValueFactsView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::portable::BoundaryWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingCarrierKindView<
        'buffer,
    >,
> for pse_modeling::resolved::CarrierKind {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingCarrierKindView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_carrier_kind::Kind::OrderedSet(
                wire,
            ) => {
                let alternative_path = child_path(path, "OrderedSet");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::CarrierKind as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::resolved::CarrierKind as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::resolved::CarrierKind as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_carrier_kind::Kind::Continuous(
                wire,
            ) => {
                let alternative_path = child_path(path, "Continuous");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::CarrierKind as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::resolved::CarrierKind as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "domain")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.domain.as_option(), &child_path(path, "domain"))?,
                        &child_path(path, "domain"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::CarrierKind as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingDeclarationKindView<
        'buffer,
    >,
> for pse_model::generated::enums::ModelingDeclarationKind {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingDeclarationKindView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingDerivedRelationshipView<
        'buffer,
    >,
> for pse_modeling::derived::Relationship {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingDerivedRelationshipView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::SubstitutedMember(
                wire,
            ) => {
                let alternative_path = child_path(path, "SubstitutedMember");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "member"), (2, "meaning")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.member,
                        &child_path(path, "member"),
                    )?,
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.meaning,
                        &child_path(path, "meaning"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::ReconstructionResultView(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReconstructionResultView");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "owner"), (2, "callee")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.owner.as_option(), &child_path(path, "owner"))?,
                        &child_path(path, "owner"),
                    )?,
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.callee,
                        &child_path(path, "callee"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::FunctionBodyInputView(
                wire,
            ) => {
                let alternative_path = child_path(path, "FunctionBodyInputView");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "owner"), (2, "ordinal"), (3, "meaning")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.owner.as_option(), &child_path(path, "owner"))?,
                        &child_path(path, "owner"),
                    )?,
                    native_from::<_, u64>(wire.ordinal, &child_path(path, "ordinal"))?,
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.meaning,
                        &child_path(path, "meaning"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::DefiningEquation(
                wire,
            ) => {
                let alternative_path = child_path(path, "DefiningEquation");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "symbol")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.symbol,
                        &child_path(path, "symbol"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::ContinuityRow(
                wire,
            ) => {
                let alternative_path = child_path(path, "ContinuityRow");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "family"), (2, "element")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.family,
                        &child_path(path, "family"),
                    )?,
                    native_from::<_, u64>(wire.element, &child_path(path, "element"))?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::ContinuousIntegral(
                wire,
            ) => {
                let alternative_path = child_path(path, "ContinuousIntegral");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "content")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.content,
                        &child_path(path, "content"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::IntegralFlux(
                wire,
            ) => {
                let alternative_path = child_path(path, "IntegralFlux");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "result")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.result,
                        &child_path(path, "result"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::ClosureConservation(
                wire,
            ) => {
                let alternative_path = child_path(path, "ClosureConservation");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "closure")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.closure,
                        &child_path(path, "closure"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::ConditionalBoundaryValue(
                wire,
            ) => {
                let alternative_path = child_path(path, "ConditionalBoundaryValue");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "symbol")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.symbol,
                        &child_path(path, "symbol"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::ConditionalBoundaryResidual(
                wire,
            ) => {
                let alternative_path = child_path(path, "ConditionalBoundaryResidual");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "symbol")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.symbol,
                        &child_path(path, "symbol"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::ConservedInventoryOutput(
                wire,
            ) => {
                let alternative_path = child_path(path, "ConservedInventoryOutput");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "balance")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.balance,
                        &child_path(path, "balance"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::InventoryTransferOutput(
                wire,
            ) => {
                let alternative_path = child_path(path, "InventoryTransferOutput");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "balance"), (2, "event")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.balance,
                        &child_path(path, "balance"),
                    )?,
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.event,
                        &child_path(path, "event"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::InitialStateOutput(
                wire,
            ) => {
                let alternative_path = child_path(path, "InitialStateOutput");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "state"), (2, "equation")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.state,
                        &child_path(path, "state"),
                    )?,
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.equation,
                        &child_path(path, "equation"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::IntegratedRateCoordinate(
                wire,
            ) => {
                let alternative_path = child_path(path, "IntegratedRateCoordinate");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "state")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.state,
                        &child_path(path, "state"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::DynamicRateOutput(
                wire,
            ) => {
                let alternative_path = child_path(path, "DynamicRateOutput");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "state")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.state,
                        &child_path(path, "state"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::TestOutput(
                wire,
            ) => {
                let alternative_path = child_path(path, "TestOutput");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "test"), (2, "component")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.test,
                        &child_path(path, "test"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTestComponentView<
                            '_,
                        >,
                    >(
                        required(
                            wire.component.as_option(),
                            &child_path(path, "component"),
                        )?,
                        &child_path(path, "component"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::AnnotationOutput(
                wire,
            ) => {
                let alternative_path = child_path(path, "AnnotationOutput");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        17,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        17,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "target"), (2, "declaration"), (3, "kind"), (4, "component")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.target,
                        &child_path(path, "target"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingAnnotationKindView<
                            '_,
                        >,
                    >(
                        required(wire.kind.as_option(), &child_path(path, "kind"))?,
                        &child_path(path, "kind"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingAnnotationComponentView<
                            '_,
                        >,
                    >(
                        required(
                            wire.component.as_option(),
                            &child_path(path, "component"),
                        )?,
                        &child_path(path, "component"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        17,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::OriginalEquationResidual(
                wire,
            ) => {
                let alternative_path = child_path(path, "OriginalEquationResidual");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        18,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        18,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "equation")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.equation,
                        &child_path(path, "equation"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        18,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::ElasticPenalty(
                wire,
            ) => {
                let alternative_path = child_path(path, "ElasticPenalty");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        19,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        19,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "equation")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.equation,
                        &child_path(path, "equation"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        19,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::OriginalTerm(
                wire,
            ) => {
                let alternative_path = child_path(path, "OriginalTerm");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        20,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        20,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "equation"), (2, "ordinal")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.equation,
                        &child_path(path, "equation"),
                    )?,
                    native_from::<_, u64>(wire.ordinal, &child_path(path, "ordinal"))?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        20,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::MemberOutput(
                wire,
            ) => {
                let alternative_path = child_path(path, "MemberOutput");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        21,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        21,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "symbol")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.symbol,
                        &child_path(path, "symbol"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        21,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::ContributionOutput(
                wire,
            ) => {
                let alternative_path = child_path(path, "ContributionOutput");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        22,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        22,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "contribution")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.contribution,
                        &child_path(path, "contribution"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        22,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::TransportAgreement(
                wire,
            ) => {
                let alternative_path = child_path(path, "TransportAgreement");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        23,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        23,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "closure")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.closure,
                        &child_path(path, "closure"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        23,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::DifferenceClosure(
                wire,
            ) => {
                let alternative_path = child_path(path, "DifferenceClosure");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        24,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        24,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "closure")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.closure,
                        &child_path(path, "closure"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        24,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::InventoryObservation(
                wire,
            ) => {
                let alternative_path = child_path(path, "InventoryObservation");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        25,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        25,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "balance")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.balance,
                        &child_path(path, "balance"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        25,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::OriginalFluxIntegral(
                wire,
            ) => {
                let alternative_path = child_path(path, "OriginalFluxIntegral");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        26,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        26,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "balance")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.balance,
                        &child_path(path, "balance"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        26,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::InventoryState(
                wire,
            ) => {
                let alternative_path = child_path(path, "InventoryState");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        27,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        27,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "balance")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.balance,
                        &child_path(path, "balance"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        27,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::InventoryInitial(
                wire,
            ) => {
                let alternative_path = child_path(path, "InventoryInitial");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        28,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        28,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "balance")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.balance,
                        &child_path(path, "balance"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        28,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::InventoryDefinition(
                wire,
            ) => {
                let alternative_path = child_path(path, "InventoryDefinition");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        29,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        29,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "balance")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.balance,
                        &child_path(path, "balance"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        29,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::InventoryRate(
                wire,
            ) => {
                let alternative_path = child_path(path, "InventoryRate");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        30,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        30,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "balance")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.balance,
                        &child_path(path, "balance"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        30,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::TemporalClosure(
                wire,
            ) => {
                let alternative_path = child_path(path, "TemporalClosure");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        31,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        31,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "closure")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.closure,
                        &child_path(path, "closure"),
                    )?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        31,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::CheckedMember(
                wire,
            ) => {
                let alternative_path = child_path(path, "CheckedMember");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        32,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        32,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "role"), (2, "ordinal")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingLocalRoleView<
                            '_,
                        >,
                    >(
                        required(wire.role.as_option(), &child_path(path, "role"))?,
                        &child_path(path, "role"),
                    )?,
                    native_from::<_, u64>(wire.ordinal, &child_path(path, "ordinal"))?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        32,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_derived_relationship::Kind::CheckedValiditySource(
                wire,
            ) => {
                let alternative_path = child_path(path, "CheckedValiditySource");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        33,
                    >>::VERSION,
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        33,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "role"), (2, "ordinal")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingLocalRoleView<
                            '_,
                        >,
                    >(
                        required(wire.role.as_option(), &child_path(path, "role"))?,
                        &child_path(path, "role"),
                    )?,
                    native_from::<_, u64>(wire.ordinal, &child_path(path, "ordinal"))?,
                );
                Ok(
                    <pse_modeling::derived::Relationship as pse_contract::boundary_projection::ProjectionAlternative<
                        33,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingEnvelopeReadFactsView<
        'buffer,
    >,
> for pse_modeling::portable::ReadsWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingEnvelopeReadFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::portable::ReadsWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::portable::ReadsWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "sets"), (2, "variables")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "sets");
                let values = wire
                    .sets
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "variables");
                let values = wire
                    .variables
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u32,
                        >(*(value), &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingLocalRoleView<
        'buffer,
    >,
> for pse_modeling::derived::LocalRole {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingLocalRoleView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_local_role::Kind::Coordinate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::LocalRole as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::derived::LocalRole as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::LocalRole as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_local_role::Kind::Parameter(
                wire,
            ) => {
                let alternative_path = child_path(path, "Parameter");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::LocalRole as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::derived::LocalRole as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::LocalRole as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_local_role::Kind::Member(
                wire,
            ) => {
                let alternative_path = child_path(path, "Member");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::LocalRole as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::derived::LocalRole as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::LocalRole as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingMemberRangeView<
        'buffer,
    >,
> for pse_modeling::resolved::MemberRange {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingMemberRangeView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::MemberRange as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::MemberRange as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "lower"), (2, "upper"), (3, "layer")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                    '_,
                >,
            >(
                required(wire.lower.as_option(), &child_path(path, "lower"))?,
                &child_path(path, "lower"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                    '_,
                >,
            >(
                required(wire.upper.as_option(), &child_path(path, "upper"))?,
                &child_path(path, "upper"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValidityLayerView<
                    '_,
                >,
            >(
                required(wire.layer.as_option(), &child_path(path, "layer"))?,
                &child_path(path, "layer"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingOperationAnchorRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::operation_records::AnchorRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingOperationAnchorRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::operation_records::AnchorRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::operation_records::AnchorRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "member"), (2, "values"), (3, "temperature"), (4, "pressure")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.member,
                &child_path(path, "member"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Array2ModelingResolvedNodeIxView<
                    '_,
                >,
            >(
                required(wire.values.as_option(), &child_path(path, "values"))?,
                &child_path(path, "values"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalCanonicalMagnitudeRecordView<
                    '_,
                >,
            >(
                required(
                    wire.temperature.as_option(),
                    &child_path(path, "temperature"),
                )?,
                &child_path(path, "temperature"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalCanonicalMagnitudeRecordView<
                    '_,
                >,
            >(
                required(wire.pressure.as_option(), &child_path(path, "pressure"))?,
                &child_path(path, "pressure"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingOperationTranslationRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::operation_records::TranslationRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingOperationTranslationRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::operation_records::TranslationRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::operation_records::TranslationRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "source"),
                (2, "target"),
                (3, "source_anchor"),
                (4, "target_anchor"),
                (5, "component_kind"),
                (6, "state"),
                (7, "provenance"),
                (8, "anchors"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                    '_,
                >,
            >(
                required(wire.source.as_option(), &child_path(path, "source"))?,
                &child_path(path, "source"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                    '_,
                >,
            >(
                required(wire.target.as_option(), &child_path(path, "target"))?,
                &child_path(path, "target"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                    '_,
                >,
            >(
                required(
                    wire.source_anchor.as_option(),
                    &child_path(path, "source_anchor"),
                )?,
                &child_path(path, "source_anchor"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                    '_,
                >,
            >(
                required(
                    wire.target_anchor.as_option(),
                    &child_path(path, "target_anchor"),
                )?,
                &child_path(path, "target_anchor"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                    '_,
                >,
            >(
                required(
                    wire.component_kind.as_option(),
                    &child_path(path, "component_kind"),
                )?,
                &child_path(path, "component_kind"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Array2ModelingResolvedNodeIxView<
                    '_,
                >,
            >(
                required(wire.state.as_option(), &child_path(path, "state"))?,
                &child_path(path, "state"),
            )?,
            {
                let current_path = &child_path(path, "provenance");
                let values = wire
                    .provenance
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "anchors");
                let values = wire
                    .anchors
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingOperationAnchorRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPermissionTargetView<
        'buffer,
    >,
> for pse_model::generated::enums::ModelingPermissionTarget {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPermissionTargetView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalAdmissionRecordView<
        'buffer,
    >,
> for pse_modeling::expression::admission::PhysicalAdmissionRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalAdmissionRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::expression::admission::PhysicalAdmissionRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::expression::admission::PhysicalAdmissionRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "request"), (2, "operands"), (3, "resolved")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalRequestFactsView<
                    '_,
                >,
            >(
                required(wire.request.as_option(), &child_path(path, "request"))?,
                &child_path(path, "request"),
            )?,
            {
                let current_path = &child_path(path, "operands");
                let values = wire
                    .operands
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .resolved
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "resolved"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalEvidenceRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::PhysicalEvidenceRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalEvidenceRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::PhysicalEvidenceRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::PhysicalEvidenceRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "identity")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.identity,
                &child_path(path, "identity"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalOperationRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::operation_records::OperationRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalOperationRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_operation_record::Kind::Coordinate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "map"), (2, "slot")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.map.as_option(), &child_path(path, "map"))?,
                        &child_path(path, "map"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.slot.as_option(), &child_path(path, "slot"))?,
                        &child_path(path, "slot"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_operation_record::Kind::ReducedLaw(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReducedLaw");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "map"), (2, "reconstruction")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.map.as_option(), &child_path(path, "map"))?,
                        &child_path(path, "map"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.reconstruction.as_option(),
                            &child_path(path, "reconstruction"),
                        )?,
                        &child_path(path, "reconstruction"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_operation_record::Kind::Reconstruction(
                wire,
            ) => {
                let alternative_path = child_path(path, "Reconstruction");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "map"), (2, "reconstruction"), (3, "reference")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.map.as_option(), &child_path(path, "map"))?,
                        &child_path(path, "map"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.reconstruction.as_option(),
                            &child_path(path, "reconstruction"),
                        )?,
                        &child_path(path, "reconstruction"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.reference.as_option(),
                            &child_path(path, "reference"),
                        )?,
                        &child_path(path, "reference"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_operation_record::Kind::Response(
                wire,
            ) => {
                let alternative_path = child_path(path, "Response");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "potential")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.potential.as_option(),
                            &child_path(path, "potential"),
                        )?,
                        &child_path(path, "potential"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_operation_record::Kind::ReferenceTranslation(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReferenceTranslation");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "translation")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingOperationTranslationRecordView<
                            '_,
                        >,
                    >(
                        required(
                            wire.translation.as_option(),
                            &child_path(path, "translation"),
                        )?,
                        &child_path(path, "translation"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_operation_record::Kind::ReferenceAnchorMean(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReferenceAnchorMean");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "translation"), (2, "source"), (3, "coefficient")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingOperationTranslationRecordView<
                            '_,
                        >,
                    >(
                        required(
                            wire.translation.as_option(),
                            &child_path(path, "translation"),
                        )?,
                        &child_path(path, "translation"),
                    )?,
                    native_from::<_, bool>(wire.source, &child_path(path, "source"))?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.coefficient.as_option(),
                            &child_path(path, "coefficient"),
                        )?,
                        &child_path(path, "coefficient"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_operation_record::Kind::Transfer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Transfer");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "source"), (2, "result"), (3, "factor"), (4, "exchange")],
                    path,
                )?;
                let fields = (
                    wire
                        .source
                        .as_option()
                        .map(|value| -> Result<_, AdapterError> {
                            native_from::<
                                _,
                                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingRefinementFactsView<
                                    '_,
                                >,
                            >(value, &child_path(path, "source"))
                        })
                        .transpose()?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingRefinementFactsView<
                            '_,
                        >,
                    >(
                        required(wire.result.as_option(), &child_path(path, "result"))?,
                        &child_path(path, "result"),
                    )?,
                    native_from::<_, i32>(wire.factor, &child_path(path, "factor"))?,
                    wire
                        .exchange
                        .as_option()
                        .map(|value| -> Result<_, AdapterError> {
                            native_from::<
                                _,
                                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                                    '_,
                                >,
                            >(value, &child_path(path, "exchange"))
                        })
                        .transpose()?,
                );
                Ok(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_operation_record::Kind::TransferMagnitude(
                wire,
            ) => {
                let alternative_path = child_path(path, "TransferMagnitude");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "source")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingRefinementFactsView<
                            '_,
                        >,
                    >(
                        required(wire.source.as_option(), &child_path(path, "source"))?,
                        &child_path(path, "source"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::operation_records::OperationRecord as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalRequestFactsView<
        'buffer,
    >,
> for pse_modeling::expression::admission::PhysicalRequest {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalRequestFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Add(
                wire,
            ) => {
                let alternative_path = child_path(path, "Add");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Sub(
                wire,
            ) => {
                let alternative_path = child_path(path, "Sub");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Neg(
                wire,
            ) => {
                let alternative_path = child_path(path, "Neg");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Abs(
                wire,
            ) => {
                let alternative_path = child_path(path, "Abs");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Mul(
                wire,
            ) => {
                let alternative_path = child_path(path, "Mul");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Div(
                wire,
            ) => {
                let alternative_path = child_path(path, "Div");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Pow(
                wire,
            ) => {
                let alternative_path = child_path(path, "Pow");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "exponent")],
                    path,
                )?;
                let fields = (
                    wire
                        .exponent
                        .as_option()
                        .map(|value| -> Result<_, AdapterError> {
                            native_from::<
                                _,
                                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
                                    '_,
                                >,
                            >(value, &child_path(path, "exponent"))
                        })
                        .transpose()?,
                );
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Sqrt(
                wire,
            ) => {
                let alternative_path = child_path(path, "Sqrt");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Transcendental(
                wire,
            ) => {
                let alternative_path = child_path(path, "Transcendental");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "opcode")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::OpcodeView<
                            '_,
                        >,
                    >(
                        required(wire.opcode.as_option(), &child_path(path, "opcode"))?,
                        &child_path(path, "opcode"),
                    )?,
                );
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::FiniteReduce(
                wire,
            ) => {
                let alternative_path = child_path(path, "FiniteReduce");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "kind"), (2, "domain")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ReductionKindView<
                            '_,
                        >,
                    >(
                        required(wire.kind.as_option(), &child_path(path, "kind"))?,
                        &child_path(path, "kind"),
                    )?,
                    wire
                        .domain
                        .as_option()
                        .map(|value| -> Result<_, AdapterError> {
                            native_from::<
                                _,
                                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::EntityKindIdView<
                                    '_,
                                >,
                            >(value, &child_path(path, "domain"))
                        })
                        .transpose()?,
                );
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Reduce(
                wire,
            ) => {
                let alternative_path = child_path(path, "Reduce");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "kind"), (2, "bound")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ReductionKindView<
                            '_,
                        >,
                    >(
                        required(wire.kind.as_option(), &child_path(path, "kind"))?,
                        &child_path(path, "kind"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BoundIndexReferenceView<
                            '_,
                        >,
                    >(
                        required(wire.bound.as_option(), &child_path(path, "bound"))?,
                        &child_path(path, "bound"),
                    )?,
                );
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_physical_request_facts::Kind::Conditional(
                wire,
            ) => {
                let alternative_path = child_path(path, "Conditional");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::VERSION,
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::expression::admission::PhysicalRequest as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingProviderSelectionView<
        'buffer,
    >,
> for pse_modeling::resolved::ProviderSelection {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingProviderSelectionView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::ProviderSelection as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::ProviderSelection as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "identity"), (2, "output")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.identity,
                &child_path(path, "identity"),
            )?,
            native_from::<_, u64>(wire.output, &child_path(path, "output"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingRefinementFactsView<
        'buffer,
    >,
> for pse_modeling::portable::RefinementWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingRefinementFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_refinement_facts::Kind::Coordinate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::RefinementWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::portable::RefinementWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "map"), (2, "slot")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.map.as_option(), &child_path(path, "map"))?,
                        &child_path(path, "map"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.slot.as_option(), &child_path(path, "slot"))?,
                        &child_path(path, "slot"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::RefinementWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_refinement_facts::Kind::ReducedLaw(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReducedLaw");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::RefinementWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::portable::RefinementWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "map"), (2, "reconstruction")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.map.as_option(), &child_path(path, "map"))?,
                        &child_path(path, "map"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.reconstruction.as_option(),
                            &child_path(path, "reconstruction"),
                        )?,
                        &child_path(path, "reconstruction"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::RefinementWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_refinement_facts::Kind::Transfer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Transfer");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::RefinementWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::portable::RefinementWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "boundary"), (2, "into")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingBoundaryFactsView<
                            '_,
                        >,
                    >(
                        required(
                            wire.boundary.as_option(),
                            &child_path(path, "boundary"),
                        )?,
                        &child_path(path, "boundary"),
                    )?,
                    native_from::<_, bool>(wire.into, &child_path(path, "into"))?,
                );
                Ok(
                    <pse_modeling::portable::RefinementWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedArgumentRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::ArgumentRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedArgumentRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::ArgumentRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::ArgumentRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "ty"),
                (2, "start"),
                (3, "len"),
                (4, "tuples"),
                (5, "domains"),
                (6, "static_value"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
                    '_,
                >,
            >(
                required(wire.ty.as_option(), &child_path(path, "ty"))?,
                &child_path(path, "ty"),
            )?,
            native_from::<_, u64>(wire.start, &child_path(path, "start"))?,
            native_from::<_, u64>(wire.len, &child_path(path, "len"))?,
            {
                let current_path = &child_path(path, "tuples");
                let values = wire
                    .tuples
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        {
                            let wrapped = value;
                            unknown_fields_view(
                                &wrapped.__buffa_unknown_fields,
                                &format!("{current_path}[{index}]"),
                            )?;
                            {
                                let current_path = &format!("{current_path}[{index}]");
                                let values = wrapped
                                    .value
                                    .iter()
                                    .enumerate()
                                    .map(|(index, value)| -> Result<_, AdapterError> {
                                        native_from::<
                                            _,
                                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                                                '_,
                                            >,
                                        >(value, &format!("{current_path}[{index}]"))
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
                let current_path = &child_path(path, "domains");
                let values = wire
                    .domains
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple3OfUsizeAndListModelingValueRecordAndModelingResolvedCarrierRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .static_value
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "static_value"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBinderIxView<
        'buffer,
    >,
> for pse_modeling::resolved::BinderIx {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBinderIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::BinderIx as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::BinderIx as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (native_from::<_, u64>(wire.value, &child_path(path, "value"))?,);
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBindingView<
        'buffer,
    >,
> for pse_modeling::resolved::Binding {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBindingView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_binding::Kind::Value(
                wire,
            ) => {
                let alternative_path = child_path(path, "Value");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_binding::Kind::Coordinate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "carrier")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierIxView<
                            '_,
                        >,
                    >(
                        required(
                            wire.carrier.as_option(),
                            &child_path(path, "carrier"),
                        )?,
                        &child_path(path, "carrier"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_binding::Kind::Accumulator(
                wire,
            ) => {
                let alternative_path = child_path(path, "Accumulator");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_binding::Kind::Item(
                wire,
            ) => {
                let alternative_path = child_path(path, "Item");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::resolved::Binding as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierIxView<
        'buffer,
    >,
> for pse_modeling::resolved::CarrierIx {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::CarrierIx as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::CarrierIx as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (native_from::<_, u64>(wire.value, &child_path(path, "value"))?,);
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::CarrierRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::CarrierRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::CarrierRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "kind"), (2, "element"), (3, "members")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingCarrierKindView<
                    '_,
                >,
            >(
                required(wire.kind.as_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
                    '_,
                >,
            >(
                required(wire.element.as_option(), &child_path(path, "element"))?,
                &child_path(path, "element"),
            )?,
            {
                let current_path = &child_path(path, "members");
                let values = wire
                    .members
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEnvelopeRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::EnvelopeRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEnvelopeRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::EnvelopeRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::EnvelopeRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "owner"), (2, "ty"), (3, "predicate"), (4, "reads")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                    '_,
                >,
            >(
                required(wire.owner.as_option(), &child_path(path, "owner"))?,
                &child_path(path, "owner"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
                    '_,
                >,
            >(
                required(wire.ty.as_option(), &child_path(path, "ty"))?,
                &child_path(path, "ty"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                    '_,
                >,
            >(
                required(wire.predicate.as_option(), &child_path(path, "predicate"))?,
                &child_path(path, "predicate"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingEnvelopeReadFactsView<
                    '_,
                >,
            >(
                required(wire.reads.as_option(), &child_path(path, "reads"))?,
                &child_path(path, "reads"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEvidenceIxView<
        'buffer,
    >,
> for pse_modeling::resolved::EvidenceIx {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEvidenceIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::EvidenceIx as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::EvidenceIx as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (native_from::<_, u64>(wire.value, &child_path(path, "value"))?,);
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEvidenceRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::EvidenceRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEvidenceRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::EvidenceRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::EvidenceRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "node"), (2, "predicates"), (3, "inputs")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ApplicabilityNodeView<
                    '_,
                >,
            >(
                required(wire.node.as_option(), &child_path(path, "node"))?,
                &child_path(path, "node"),
            )?,
            {
                let current_path = &child_path(path, "predicates");
                let values = wire
                    .predicates
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedExternalRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::ExternalRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedExternalRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::ExternalRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::ExternalRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "implementation"),
                (2, "revision"),
                (3, "data"),
                (4, "output"),
                (5, "derivative_source"),
                (6, "derivatives"),
                (7, "smoothness"),
                (8, "shapes"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &str,
            >(wire.implementation, &child_path(path, "implementation"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.revision,
                &child_path(path, "revision"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.data,
                &child_path(path, "data"),
            )?,
            native_from::<_, u64>(wire.output, &child_path(path, "output"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ExternalDerivativeSourceView<
                    '_,
                >,
            >(
                required(
                    wire.derivative_source.as_option(),
                    &child_path(path, "derivative_source"),
                )?,
                &child_path(path, "derivative_source"),
            )?,
            native_from::<_, u32>(wire.derivatives, &child_path(path, "derivatives"))?,
            native_from::<_, u32>(wire.smoothness, &child_path(path, "smoothness"))?,
            {
                let current_path = &child_path(path, "shapes");
                let values = wire
                    .shapes
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedShapeRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFoldRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::FoldRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFoldRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::FoldRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::FoldRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "root"), (2, "carrier"), (3, "selected"), (4, "prototype")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                    '_,
                >,
            >(
                required(wire.root.as_option(), &child_path(path, "root"))?,
                &child_path(path, "root"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierIxView<
                    '_,
                >,
            >(
                required(wire.carrier.as_option(), &child_path(path, "carrier"))?,
                &child_path(path, "carrier"),
            )?,
            {
                let current_path = &child_path(path, "selected");
                let values = wire
                    .selected
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
                    '_,
                >,
            >(
                required(wire.prototype.as_option(), &child_path(path, "prototype"))?,
                &child_path(path, "prototype"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionIxView<
        'buffer,
    >,
> for pse_modeling::resolved::FunctionIx {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::FunctionIx as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::FunctionIx as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (native_from::<_, u64>(wire.value, &child_path(path, "value"))?,);
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::FunctionRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::FunctionRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::FunctionRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "identity"),
                (2, "declaration"),
                (3, "syntax"),
                (4, "physical"),
                (5, "formals"),
                (6, "signature"),
                (7, "arguments"),
                (8, "selections"),
                (9, "literals"),
                (10, "result"),
                (11, "body"),
                (12, "validity"),
                (13, "envelopes"),
                (14, "applicability"),
                (15, "prerequisites"),
                (16, "admissions"),
                (17, "reduction_admission"),
                (18, "reduction"),
                (19, "reduction_selection"),
                (20, "physical_operation"),
                (21, "output_boundary"),
                (22, "body_inputs"),
                (23, "validity_reads"),
                (24, "external"),
                (25, "continuity"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.identity,
                &child_path(path, "identity"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                    '_,
                >,
            >(
                required(
                    wire.declaration.as_option(),
                    &child_path(path, "declaration"),
                )?,
                &child_path(path, "declaration"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSyntaxRecordView<
                    '_,
                >,
            >(
                required(wire.syntax.as_option(), &child_path(path, "syntax"))?,
                &child_path(path, "syntax"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportRecordView<
                    '_,
                >,
            >(
                required(wire.physical.as_option(), &child_path(path, "physical"))?,
                &child_path(path, "physical"),
            )?,
            {
                let current_path = &child_path(path, "formals");
                let values = wire
                    .formals
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSymbolIxView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "signature");
                let values = wire
                    .signature
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "arguments");
                let values = wire
                    .arguments
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedArgumentRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "selections");
                let values = wire
                    .selections
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSelectionRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "literals");
                let values = wire
                    .literals
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingResolvedNodeIxAndModelingTypeRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
                    '_,
                >,
            >(
                required(wire.result.as_option(), &child_path(path, "result"))?,
                &child_path(path, "result"),
            )?,
            wire
                .body
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(value, &child_path(path, "body"))
                })
                .transpose()?,
            wire
                .validity
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                            '_,
                        >,
                    >(value, &child_path(path, "validity"))
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "envelopes");
                let values = wire
                    .envelopes
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEnvelopeRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "applicability");
                let values = wire
                    .applicability
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEvidenceRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "prerequisites");
                let values = wire
                    .prerequisites
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(*(value), &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "admissions");
                let values = wire
                    .admissions
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingResolvedNodeIxAndModelingPhysicalAdmissionRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .reduction_admission
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalAdmissionRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "reduction_admission"))
                })
                .transpose()?,
            wire
                .reduction
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedReductionRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "reduction"))
                })
                .transpose()?,
            wire
                .reduction_selection
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedReductionSelectionRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "reduction_selection"))
                })
                .transpose()?,
            wire
                .physical_operation
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalOperationRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "physical_operation"))
                })
                .transpose()?,
            wire
                .output_boundary
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "output_boundary"))
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "body_inputs");
                let values = wire
                    .body_inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        {
                            let wrapped = value;
                            unknown_fields_view(
                                &wrapped.__buffa_unknown_fields,
                                &format!("{current_path}[{index}]"),
                            )?;
                            wrapped
                                .value
                                .as_option()
                                .map(|value| -> Result<_, AdapterError> {
                                    native_from::<
                                        _,
                                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingBodyInputRecordView<
                                            '_,
                                        >,
                                    >(value, &format!("{current_path}[{index}]"))
                                })
                                .transpose()
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingEnvelopeReadFactsView<
                    '_,
                >,
            >(
                required(
                    wire.validity_reads.as_option(),
                    &child_path(path, "validity_reads"),
                )?,
                &child_path(path, "validity_reads"),
            )?,
            wire
                .external
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedExternalRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "external"))
                })
                .transpose()?,
            wire
                .continuity
                .as_ref()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u32>(*(value), &child_path(path, "continuity"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionSlotView<
        'buffer,
    >,
> for pse_modeling::resolved::records::FunctionSlot {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionSlotView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::FunctionSlot as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::FunctionSlot as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "identity"), (2, "declaration"), (3, "formal_count")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.identity,
                &child_path(path, "identity"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                    '_,
                >,
            >(
                required(
                    wire.declaration.as_option(),
                    &child_path(path, "declaration"),
                )?,
                &child_path(path, "declaration"),
            )?,
            native_from::<_, u64>(wire.formal_count, &child_path(path, "formal_count"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedGroupIxView<
        'buffer,
    >,
> for pse_modeling::resolved::GroupIx {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedGroupIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::GroupIx as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::GroupIx as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (native_from::<_, u64>(wire.value, &child_path(path, "value"))?,);
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedGroupRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::GroupRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedGroupRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::GroupRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::GroupRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "axes"), (2, "members")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "axes");
                let values = wire
                    .axes
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierIxView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "members");
                let values = wire
                    .members
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfListModelingValueRecordAndOptionalModelingResolvedSymbolIxView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeView<
        'buffer,
    >,
> for pse_modeling::resolved::Node {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Number(
                wire,
            ) => {
                let alternative_path = child_path(path, "Number");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "number")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredScientificNumberView<
                            '_,
                        >,
                    >(
                        required(wire.number.as_option(), &child_path(path, "number"))?,
                        &child_path(path, "number"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Reference(
                wire,
            ) => {
                let alternative_path = child_path(path, "Reference");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "reference")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedReferenceView<
                            '_,
                        >,
                    >(
                        required(
                            wire.reference.as_option(),
                            &child_path(path, "reference"),
                        )?,
                        &child_path(path, "reference"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Neg(
                wire,
            ) => {
                let alternative_path = child_path(path, "Neg");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Binary(
                wire,
            ) => {
                let alternative_path = child_path(path, "Binary");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "op"), (2, "lhs"), (3, "rhs")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredBinaryOperatorView<
                            '_,
                        >,
                    >(
                        required(wire.op.as_option(), &child_path(path, "op"))?,
                        &child_path(path, "op"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.lhs.as_option(), &child_path(path, "lhs"))?,
                        &child_path(path, "lhs"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.rhs.as_option(), &child_path(path, "rhs"))?,
                        &child_path(path, "rhs"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Call(
                wire,
            ) => {
                let alternative_path = child_path(path, "Call");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "function"), (2, "args")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredBuiltinFunctionView<
                            '_,
                        >,
                    >(
                        required(
                            wire.function.as_option(),
                            &child_path(path, "function"),
                        )?,
                        &child_path(path, "function"),
                    )?,
                    {
                        let current_path = &child_path(path, "args");
                        let values = wire
                            .args
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Apply(
                wire,
            ) => {
                let alternative_path = child_path(path, "Apply");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "function"), (2, "args")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionIxView<
                            '_,
                        >,
                    >(
                        required(
                            wire.function.as_option(),
                            &child_path(path, "function"),
                        )?,
                        &child_path(path, "function"),
                    )?,
                    {
                        let current_path = &child_path(path, "args");
                        let values = wire
                            .args
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Partial(
                wire,
            ) => {
                let alternative_path = child_path(path, "Partial");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "function"), (2, "wrt"), (3, "args")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionIxView<
                            '_,
                        >,
                    >(
                        required(
                            wire.function.as_option(),
                            &child_path(path, "function"),
                        )?,
                        &child_path(path, "function"),
                    )?,
                    {
                        let current_path = &child_path(path, "wrt");
                        let values = wire
                            .wrt
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    u64,
                                >(*(value), &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                    {
                        let current_path = &child_path(path, "args");
                        let values = wire
                            .args
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Provider(
                wire,
            ) => {
                let alternative_path = child_path(path, "Provider");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "selection"), (2, "args")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingProviderSelectionView<
                            '_,
                        >,
                    >(
                        required(
                            wire.selection.as_option(),
                            &child_path(path, "selection"),
                        )?,
                        &child_path(path, "selection"),
                    )?,
                    {
                        let current_path = &child_path(path, "args");
                        let values = wire
                            .args
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Reduce(
                wire,
            ) => {
                let alternative_path = child_path(path, "Reduce");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "kind"), (2, "binder"), (3, "filter"), (4, "body")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredReductionKindView<
                            '_,
                        >,
                    >(
                        required(wire.kind.as_option(), &child_path(path, "kind"))?,
                        &child_path(path, "kind"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBinderIxView<
                            '_,
                        >,
                    >(
                        required(wire.binder.as_option(), &child_path(path, "binder"))?,
                        &child_path(path, "binder"),
                    )?,
                    wire
                        .filter
                        .as_option()
                        .map(|value| -> Result<_, AdapterError> {
                            native_from::<
                                _,
                                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                                    '_,
                                >,
                            >(value, &child_path(path, "filter"))
                        })
                        .transpose()?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.body.as_option(), &child_path(path, "body"))?,
                        &child_path(path, "body"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Sequence(
                wire,
            ) => {
                let alternative_path = child_path(path, "Sequence");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value"), (2, "prerequisites")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                    {
                        let current_path = &child_path(path, "prerequisites");
                        let values = wire
                            .prerequisites
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Evidence(
                wire,
            ) => {
                let alternative_path = child_path(path, "Evidence");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value"), (2, "evidence")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEvidenceIxView<
                            '_,
                        >,
                    >(
                        required(
                            wire.evidence.as_option(),
                            &child_path(path, "evidence"),
                        )?,
                        &child_path(path, "evidence"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Fold(
                wire,
            ) => {
                let alternative_path = child_path(path, "Fold");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::FIELD_LAYOUT,
                    1,
                    &[
                        (1, "binder"),
                        (2, "filter"),
                        (3, "value"),
                        (4, "accumulator"),
                        (5, "item"),
                        (6, "step"),
                    ],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBinderIxView<
                            '_,
                        >,
                    >(
                        required(wire.binder.as_option(), &child_path(path, "binder"))?,
                        &child_path(path, "binder"),
                    )?,
                    wire
                        .filter
                        .as_option()
                        .map(|value| -> Result<_, AdapterError> {
                            native_from::<
                                _,
                                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                                    '_,
                                >,
                            >(value, &child_path(path, "filter"))
                        })
                        .transpose()?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBinderIxView<
                            '_,
                        >,
                    >(
                        required(
                            wire.accumulator.as_option(),
                            &child_path(path, "accumulator"),
                        )?,
                        &child_path(path, "accumulator"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBinderIxView<
                            '_,
                        >,
                    >(
                        required(wire.item.as_option(), &child_path(path, "item"))?,
                        &child_path(path, "item"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.step.as_option(), &child_path(path, "step"))?,
                        &child_path(path, "step"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Derivative(
                wire,
            ) => {
                let alternative_path = child_path(path, "Derivative");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "body"), (2, "domain")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.body.as_option(), &child_path(path, "body"))?,
                        &child_path(path, "body"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierIxView<
                            '_,
                        >,
                    >(
                        required(wire.domain.as_option(), &child_path(path, "domain"))?,
                        &child_path(path, "domain"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Indicator(
                wire,
            ) => {
                let alternative_path = child_path(path, "Indicator");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "predicate")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                            '_,
                        >,
                    >(
                        required(
                            wire.predicate.as_option(),
                            &child_path(path, "predicate"),
                        )?,
                        &child_path(path, "predicate"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Conditional(
                wire,
            ) => {
                let alternative_path = child_path(path, "Conditional");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "guard"), (2, "then"), (3, "otherwise")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                            '_,
                        >,
                    >(
                        required(wire.guard.as_option(), &child_path(path, "guard"))?,
                        &child_path(path, "guard"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.then.as_option(), &child_path(path, "then"))?,
                        &child_path(path, "then"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(
                            wire.otherwise.as_option(),
                            &child_path(path, "otherwise"),
                        )?,
                        &child_path(path, "otherwise"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_node::Kind::Let(
                wire,
            ) => {
                let alternative_path = child_path(path, "Let");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::VERSION,
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "bindings"), (2, "body")],
                    path,
                )?;
                let fields = (
                    {
                        let current_path = &child_path(path, "bindings");
                        let values = wire
                            .bindings
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBinderIxView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.body.as_option(), &child_path(path, "body"))?,
                        &child_path(path, "body"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Node as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
        'buffer,
    >,
> for pse_modeling::resolved::NodeIx {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::NodeIx as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::NodeIx as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (native_from::<_, u64>(wire.value, &child_path(path, "value"))?,);
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateView<
        'buffer,
    >,
> for pse_modeling::resolved::ResolvedPredicate {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_predicate::Kind::Compare(
                wire,
            ) => {
                let alternative_path = child_path(path, "Compare");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "op"), (2, "lhs"), (3, "rhs")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::AuthoredComparisonOperatorView<
                            '_,
                        >,
                    >(
                        required(wire.op.as_option(), &child_path(path, "op"))?,
                        &child_path(path, "op"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.lhs.as_option(), &child_path(path, "lhs"))?,
                        &child_path(path, "lhs"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.rhs.as_option(), &child_path(path, "rhs"))?,
                        &child_path(path, "rhs"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_predicate::Kind::In(
                wire,
            ) => {
                let alternative_path = child_path(path, "In");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value"), (2, "domain")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierIxView<
                            '_,
                        >,
                    >(
                        required(wire.domain.as_option(), &child_path(path, "domain"))?,
                        &child_path(path, "domain"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_predicate::Kind::And(
                wire,
            ) => {
                let alternative_path = child_path(path, "And");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "left"), (2, "right")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                            '_,
                        >,
                    >(
                        required(wire.left.as_option(), &child_path(path, "left"))?,
                        &child_path(path, "left"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                            '_,
                        >,
                    >(
                        required(wire.right.as_option(), &child_path(path, "right"))?,
                        &child_path(path, "right"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_predicate::Kind::Or(
                wire,
            ) => {
                let alternative_path = child_path(path, "Or");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "left"), (2, "right")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                            '_,
                        >,
                    >(
                        required(wire.left.as_option(), &child_path(path, "left"))?,
                        &child_path(path, "left"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                            '_,
                        >,
                    >(
                        required(wire.right.as_option(), &child_path(path, "right"))?,
                        &child_path(path, "right"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_predicate::Kind::Not(
                wire,
            ) => {
                let alternative_path = child_path(path, "Not");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_predicate::Kind::Atom(
                wire,
            ) => {
                let alternative_path = child_path(path, "Atom");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_predicate::Kind::Bool(
                wire,
            ) => {
                let alternative_path = child_path(path, "Bool");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, bool>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_predicate::Kind::Null(
                wire,
            ) => {
                let alternative_path = child_path(path, "Null");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::resolved::ResolvedPredicate as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
        'buffer,
    >,
> for pse_modeling::resolved::PredicateIx {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::PredicateIx as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::PredicateIx as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (native_from::<_, u64>(wire.value, &child_path(path, "value"))?,);
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::ResolvedRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::ResolvedRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::ResolvedRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "syntax"), (2, "functions")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSyntaxRecordView<
                    '_,
                >,
            >(
                required(wire.syntax.as_option(), &child_path(path, "syntax"))?,
                &child_path(path, "syntax"),
            )?,
            {
                let current_path = &child_path(path, "functions");
                let values = wire
                    .functions
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedReductionRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::ReductionRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedReductionRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::ReductionRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::ReductionRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "kind"), (2, "domain"), (3, "prototype")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ReductionKindView<
                    '_,
                >,
            >(
                required(wire.kind.as_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            wire
                .domain
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::EntityKindIdView<
                            '_,
                        >,
                    >(value, &child_path(path, "domain"))
                })
                .transpose()?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                    '_,
                >,
            >(
                required(wire.prototype.as_option(), &child_path(path, "prototype"))?,
                &child_path(path, "prototype"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedReductionSelectionRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::ReductionSelectionRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedReductionSelectionRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::ReductionSelectionRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::ReductionSelectionRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "carrier"), (2, "selected")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierRecordView<
                    '_,
                >,
            >(
                required(wire.carrier.as_option(), &child_path(path, "carrier"))?,
                &child_path(path, "carrier"),
            )?,
            {
                let current_path = &child_path(path, "selected");
                let values = wire
                    .selected
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedReferenceView<
        'buffer,
    >,
> for pse_modeling::resolved::Reference {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedReferenceView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_reference::Kind::Symbol(
                wire,
            ) => {
                let alternative_path = child_path(path, "Symbol");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Reference as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::resolved::Reference as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "symbol")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSymbolIxView<
                            '_,
                        >,
                    >(
                        required(wire.symbol.as_option(), &child_path(path, "symbol"))?,
                        &child_path(path, "symbol"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Reference as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_reference::Kind::Lexical(
                wire,
            ) => {
                let alternative_path = child_path(path, "Lexical");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Reference as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::resolved::Reference as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "binder")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBinderIxView<
                            '_,
                        >,
                    >(
                        required(wire.binder.as_option(), &child_path(path, "binder"))?,
                        &child_path(path, "binder"),
                    )?,
                );
                Ok(
                    <pse_modeling::resolved::Reference as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_resolved_reference::Kind::Indexed(
                wire,
            ) => {
                let alternative_path = child_path(path, "Indexed");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::resolved::Reference as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::resolved::Reference as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "group"), (2, "indices")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedGroupIxView<
                            '_,
                        >,
                    >(
                        required(wire.group.as_option(), &child_path(path, "group"))?,
                        &child_path(path, "group"),
                    )?,
                    {
                        let current_path = &child_path(path, "indices");
                        let values = wire
                            .indices
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::resolved::Reference as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedScalarRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::ScalarRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedScalarRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::ScalarRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::ScalarRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "ty"), (3, "role"), (4, "domain"), (5, "definition")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.id,
                &child_path(path, "id"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
                    '_,
                >,
            >(
                required(wire.ty.as_option(), &child_path(path, "ty"))?,
                &child_path(path, "ty"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingDeclarationKindView<
                    '_,
                >,
            >(
                required(wire.role.as_option(), &child_path(path, "role"))?,
                &child_path(path, "role"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingVariableDomainView<
                    '_,
                >,
            >(
                required(wire.domain.as_option(), &child_path(path, "domain"))?,
                &child_path(path, "domain"),
            )?,
            wire
                .definition
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                            '_,
                        >,
                    >(value, &child_path(path, "definition"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSelectedRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::SelectedRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSelectedRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::SelectedRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::SelectedRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "kind"), (2, "origin"), (3, "values"), (4, "uncertainties")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                    '_,
                >,
            >(
                required(wire.kind.as_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                    '_,
                >,
            >(
                required(wire.origin.as_option(), &child_path(path, "origin"))?,
                &child_path(path, "origin"),
            )?,
            {
                let current_path = &child_path(path, "values");
                let values = wire
                    .values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfStringAndModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "uncertainties");
                let values = wire
                    .uncertainties
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple3OfStringAndModelingUncertaintyKindAndU64View<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSelectionRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::SelectionRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSelectionRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::SelectionRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::SelectionRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "dependencies"),
                (2, "roots"),
                (3, "context"),
                (4, "records"),
                (5, "edges"),
                (6, "selected"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                    '_,
                >,
            >(
                required(
                    wire.dependencies.as_option(),
                    &child_path(path, "dependencies"),
                )?,
                &child_path(path, "dependencies"),
            )?,
            {
                let current_path = &child_path(path, "roots");
                let values = wire
                    .roots
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                    '_,
                >,
            >(
                required(wire.context.as_option(), &child_path(path, "context"))?,
                &child_path(path, "context"),
            )?,
            {
                let current_path = &child_path(path, "records");
                let values = wire
                    .records
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "edges");
                let values = wire
                    .edges
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingValueRecordAndListModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "selected");
                let values = wire
                    .selected
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSelectedRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedShapeRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::ShapeRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedShapeRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::ShapeRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::ShapeRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "argument"), (2, "axes"), (3, "coordinates"), (4, "start")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.argument,
                &child_path(path, "argument"),
            )?,
            {
                let current_path = &child_path(path, "axes");
                let values = wire
                    .axes
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "coordinates");
                let values = wire
                    .coordinates
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        {
                            let wrapped = value;
                            unknown_fields_view(
                                &wrapped.__buffa_unknown_fields,
                                &format!("{current_path}[{index}]"),
                            )?;
                            {
                                let current_path = &format!("{current_path}[{index}]");
                                let values = wrapped
                                    .value
                                    .iter()
                                    .enumerate()
                                    .map(|(index, value)| -> Result<_, AdapterError> {
                                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                                            value,
                                            &format!("{current_path}[{index}]"),
                                        )
                                    })
                                    .collect::<Result<Vec<_>, _>>()?;
                                from_items(values, current_path)
                            }
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u64>(wire.start, &child_path(path, "start"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSymbolIxView<
        'buffer,
    >,
> for pse_modeling::resolved::SymbolIx {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSymbolIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::SymbolIx as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::SymbolIx as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (native_from::<_, u64>(wire.value, &child_path(path, "value"))?,);
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSyntaxRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::records::SyntaxRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSyntaxRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::records::SyntaxRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::records::SyntaxRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "identity"),
                (2, "symbols"),
                (3, "nodes"),
                (4, "predicates"),
                (5, "binders"),
                (6, "functions"),
                (7, "evidence"),
                (8, "carriers"),
                (9, "groups"),
                (10, "member_ranges"),
                (11, "finite_folds"),
                (12, "outputs"),
                (13, "validity"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.identity,
                &child_path(path, "identity"),
            )?,
            {
                let current_path = &child_path(path, "symbols");
                let values = wire
                    .symbols
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedScalarRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "nodes");
                let values = wire
                    .nodes
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "predicates");
                let values = wire
                    .predicates
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "binders");
                let values = wire
                    .binders
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedBindingView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "functions");
                let values = wire
                    .functions
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFunctionSlotView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "evidence");
                let values = wire
                    .evidence
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedEvidenceRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "carriers");
                let values = wire
                    .carriers
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "groups");
                let values = wire
                    .groups
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedGroupRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "member_ranges");
                let values = wire
                    .member_ranges
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingResolvedSymbolIxAndModelingMemberRangeView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "finite_folds");
                let values = wire
                    .finite_folds
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedFoldRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "outputs");
                let values = wire
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "validity");
                let values = wire
                    .validity
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedPredicateIxView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingStaticValueFactsView<
        'buffer,
    >,
> for pse_modeling::portable::ValueWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingStaticValueFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Boolean(
                wire,
            ) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Integer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Number(
                wire,
            ) => {
                let alternative_path = child_path(path, "Number");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
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
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.quantity.as_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Coordinate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "bits"), (3, "quantity")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.id,
                        &child_path(path, "id"),
                    )?,
                    native_from::<_, u64>(wire.bits, &child_path(path, "bits"))?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.quantity.as_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Text(
                wire,
            ) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<_, &str>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Entity(
                wire,
            ) => {
                let alternative_path = child_path(path, "Entity");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "kind")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.id.as_option(), &child_path(path, "id"))?,
                        &child_path(path, "id"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.kind.as_option(), &child_path(path, "kind"))?,
                        &child_path(path, "kind"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Enum(
                wire,
            ) => {
                let alternative_path = child_path(path, "Enum");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "enumeration"), (2, "member")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.enumeration.as_option(),
                            &child_path(path, "enumeration"),
                        )?,
                        &child_path(path, "enumeration"),
                    )?,
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                        wire.member,
                        &child_path(path, "member"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Identifier(
                wire,
            ) => {
                let alternative_path = child_path(path, "Identifier");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "scheme"), (2, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.scheme.as_option(), &child_path(path, "scheme"))?,
                        &child_path(path, "scheme"),
                    )?,
                    native_from::<_, &str>(wire.value, &child_path(path, "value"))?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Definition(
                wire,
            ) => {
                let alternative_path = child_path(path, "Definition");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "bindings")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.id.as_option(), &child_path(path, "id"))?,
                        &child_path(path, "id"),
                    )?,
                    unique_pairs(
                        wire
                            .bindings
                            .iter_unique()
                            .map(|(key, value)| -> Result<_, AdapterError> {
                                Ok((
                                    native_from::<_, &str>(key, &child_path(path, "bindings"))?,
                                    native_from::<
                                        _,
                                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingStaticValueFactsView<
                                            '_,
                                        >,
                                    >(value, &child_path(path, "bindings"))?,
                                ))
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                        &child_path(path, "bindings"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Function(
                wire,
            ) => {
                let alternative_path = child_path(path, "Function");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Row(
                wire,
            ) => {
                let alternative_path = child_path(path, "Row");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "table"), (2, "names"), (3, "fields")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.table.as_option(), &child_path(path, "table"))?,
                        &child_path(path, "table"),
                    )?,
                    {
                        let current_path = &child_path(path, "names");
                        let values = wire
                            .names
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &str,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                    {
                        let current_path = &child_path(path, "fields");
                        let values = wire
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingStaticValueFactsView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Set(
                wire,
            ) => {
                let alternative_path = child_path(path, "Set");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "values")],
                    path,
                )?;
                let fields = (
                    {
                        let current_path = &child_path(path, "values");
                        let values = wire
                            .values
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingStaticValueFactsView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Tuple(
                wire,
            ) => {
                let alternative_path = child_path(path, "Tuple");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "values")],
                    path,
                )?;
                let fields = (
                    {
                        let current_path = &child_path(path, "values");
                        let values = wire
                            .values
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingStaticValueFactsView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::QuantityType(
                wire,
            ) => {
                let alternative_path = child_path(path, "QuantityType");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "quantity")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.quantity.as_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::ReferenceState(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReferenceState");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "reference")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ReferenceStateIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.reference.as_option(),
                            &child_path(path, "reference"),
                        )?,
                        &child_path(path, "reference"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_static_value_facts::Kind::Missing(
                wire,
            ) => {
                let alternative_path = child_path(path, "Missing");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::VERSION,
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::portable::ValueWire as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportBooleanRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::support_records::BooleanRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportBooleanRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::support_records::BooleanRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::support_records::BooleanRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "contract"), (2, "conversion"), (3, "literal"), (4, "conditional")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                    '_,
                >,
            >(
                required(wire.contract.as_option(), &child_path(path, "contract"))?,
                &child_path(path, "contract"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalCanonicalConversionRecordView<
                    '_,
                >,
            >(
                required(wire.conversion.as_option(), &child_path(path, "conversion"))?,
                &child_path(path, "conversion"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
                    '_,
                >,
            >(
                required(wire.literal.as_option(), &child_path(path, "literal"))?,
                &child_path(path, "literal"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
                    '_,
                >,
            >(
                required(
                    wire.conditional.as_option(),
                    &child_path(path, "conditional"),
                )?,
                &child_path(path, "conditional"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportCallRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::support_records::CallRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportCallRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::support_records::CallRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::support_records::CallRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "function"),
                (2, "nominal_result"),
                (3, "wrt"),
                (4, "inputs"),
                (5, "body_result"),
                (6, "partial"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.function, &child_path(path, "function"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                    '_,
                >,
            >(
                required(
                    wire.nominal_result.as_option(),
                    &child_path(path, "nominal_result"),
                )?,
                &child_path(path, "nominal_result"),
            )?,
            {
                let current_path = &child_path(path, "wrt");
                let values = wire
                    .wrt
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(*(value), &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .body_result
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingBodyInputRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "body_result"))
                })
                .transpose()?,
            wire
                .partial
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "partial"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportNodeRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::support_records::NodeRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportNodeRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::support_records::NodeRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::support_records::NodeRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "slot"),
                (2, "ty"),
                (3, "contract"),
                (4, "operation"),
                (5, "literal"),
                (6, "boundary"),
                (7, "call"),
                (8, "range"),
                (9, "exponent"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.slot, &child_path(path, "slot"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
                    '_,
                >,
            >(
                required(wire.ty.as_option(), &child_path(path, "ty"))?,
                &child_path(path, "ty"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                    '_,
                >,
            >(
                required(wire.contract.as_option(), &child_path(path, "contract"))?,
                &child_path(path, "contract"),
            )?,
            wire
                .operation
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "operation"))
                })
                .transpose()?,
            wire
                .literal
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalCanonicalConversionRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "literal"))
                })
                .transpose()?,
            wire
                .boundary
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "boundary"))
                })
                .transpose()?,
            wire
                .call
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportCallRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "call"))
                })
                .transpose()?,
            wire
                .range
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportRangeRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "range"))
                })
                .transpose()?,
            wire
                .exponent
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
                            '_,
                        >,
                    >(value, &child_path(path, "exponent"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportRangeRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::support_records::RangeRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportRangeRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::support_records::RangeRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::support_records::RangeRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "lower"),
                (2, "upper"),
                (3, "lower_difference"),
                (4, "upper_difference"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
                    '_,
                >,
            >(
                required(wire.lower.as_option(), &child_path(path, "lower"))?,
                &child_path(path, "lower"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
                    '_,
                >,
            >(
                required(wire.upper.as_option(), &child_path(path, "upper"))?,
                &child_path(path, "upper"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
                    '_,
                >,
            >(
                required(
                    wire.lower_difference.as_option(),
                    &child_path(path, "lower_difference"),
                )?,
                &child_path(path, "lower_difference"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
                    '_,
                >,
            >(
                required(
                    wire.upper_difference.as_option(),
                    &child_path(path, "upper_difference"),
                )?,
                &child_path(path, "upper_difference"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportRecordView<
        'buffer,
    >,
> for pse_modeling::resolved::SupportRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::SupportRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::SupportRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "identity"),
                (2, "syntax"),
                (3, "nodes"),
                (4, "outputs"),
                (5, "inputs"),
                (6, "premises"),
                (7, "boolean"),
                (8, "reduction"),
                (9, "reduction_output"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.identity,
                &child_path(path, "identity"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.syntax,
                &child_path(path, "syntax"),
            )?,
            {
                let current_path = &child_path(path, "nodes");
                let values = wire
                    .nodes
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportNodeRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "outputs");
                let values = wire
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfUsizeAndPhysicalContractRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalEvidenceRecordView<
                    '_,
                >,
            >(
                required(wire.premises.as_option(), &child_path(path, "premises"))?,
                &child_path(path, "premises"),
            )?,
            wire
                .boolean
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportBooleanRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "boolean"))
                })
                .transpose()?,
            wire
                .reduction
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "reduction"))
                })
                .transpose()?,
            wire
                .reduction_output
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
                            '_,
                        >,
                    >(value, &child_path(path, "reduction_output"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTestComponentView<
        'buffer,
    >,
> for pse_modeling::derived::TestComponent {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTestComponentView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_test_component::Kind::Actual(
                wire,
            ) => {
                let alternative_path = child_path(path, "Actual");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_test_component::Kind::Expected(
                wire,
            ) => {
                let alternative_path = child_path(path, "Expected");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_test_component::Kind::Tolerance(
                wire,
            ) => {
                let alternative_path = child_path(path, "Tolerance");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_test_component::Kind::RelativeTolerance(
                wire,
            ) => {
                let alternative_path = child_path(path, "RelativeTolerance");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::derived::TestComponent as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
        'buffer,
    >,
> for pse_modeling::portable::TypeWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Boolean(
                wire,
            ) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Integer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Text(
                wire,
            ) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::QuantityType(
                wire,
            ) => {
                let alternative_path = child_path(path, "QuantityType");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::ReferenceState(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReferenceState");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Applicability(
                wire,
            ) => {
                let alternative_path = child_path(path, "Applicability");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Boundary(
                wire,
            ) => {
                let alternative_path = child_path(path, "Boundary");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::CoordinateMap(
                wire,
            ) => {
                let alternative_path = child_path(path, "CoordinateMap");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Reconstruction(
                wire,
            ) => {
                let alternative_path = child_path(path, "Reconstruction");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration"), (2, "map")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.map.as_option(), &child_path(path, "map"))?,
                        &child_path(path, "map"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Quantity(
                wire,
            ) => {
                let alternative_path = child_path(path, "Quantity");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "scheme")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeRecordView<
                            '_,
                        >,
                    >(
                        required(wire.scheme.as_option(), &child_path(path, "scheme"))?,
                        &child_path(path, "scheme"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Refined(
                wire,
            ) => {
                let alternative_path = child_path(path, "Refined");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "quantity"), (2, "refinement")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeRecordView<
                            '_,
                        >,
                    >(
                        required(
                            wire.quantity.as_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingRefinementFactsView<
                            '_,
                        >,
                    >(
                        required(
                            wire.refinement.as_option(),
                            &child_path(path, "refinement"),
                        )?,
                        &child_path(path, "refinement"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Entity(
                wire,
            ) => {
                let alternative_path = child_path(path, "Entity");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Enum(
                wire,
            ) => {
                let alternative_path = child_path(path, "Enum");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Identifier(
                wire,
            ) => {
                let alternative_path = child_path(path, "Identifier");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Set(
                wire,
            ) => {
                let alternative_path = child_path(path, "Set");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "element")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
                            '_,
                        >,
                    >(
                        required(
                            wire.element.as_option(),
                            &child_path(path, "element"),
                        )?,
                        &child_path(path, "element"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Continuous(
                wire,
            ) => {
                let alternative_path = child_path(path, "Continuous");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "domain"), (2, "element")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(wire.domain.as_option(), &child_path(path, "domain"))?,
                        &child_path(path, "domain"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
                            '_,
                        >,
                    >(
                        required(
                            wire.element.as_option(),
                            &child_path(path, "element"),
                        )?,
                        &child_path(path, "element"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Tuple(
                wire,
            ) => {
                let alternative_path = child_path(path, "Tuple");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        17,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        17,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "elements")],
                    path,
                )?;
                let fields = (
                    {
                        let current_path = &child_path(path, "elements");
                        let values = wire
                            .elements
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        17,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Table(
                wire,
            ) => {
                let alternative_path = child_path(path, "Table");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        18,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        18,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        18,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Row(
                wire,
            ) => {
                let alternative_path = child_path(path, "Row");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        19,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        19,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        19,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Definition(
                wire,
            ) => {
                let alternative_path = child_path(path, "Definition");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        20,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        20,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        20,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Interface(
                wire,
            ) => {
                let alternative_path = child_path(path, "Interface");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        21,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        21,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.declaration.as_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        21,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Function(
                wire,
            ) => {
                let alternative_path = child_path(path, "Function");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        22,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        22,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "arguments"), (2, "result")],
                    path,
                )?;
                let fields = (
                    {
                        let current_path = &child_path(path, "arguments");
                        let values = wire
                            .arguments
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfStringAndModelingTypeFactsView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
                            '_,
                        >,
                    >(
                        required(wire.result.as_option(), &child_path(path, "result"))?,
                        &child_path(path, "result"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        22,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Optional(
                wire,
            ) => {
                let alternative_path = child_path(path, "Optional");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        23,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        23,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "element")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
                            '_,
                        >,
                    >(
                        required(
                            wire.element.as_option(),
                            &child_path(path, "element"),
                        )?,
                        &child_path(path, "element"),
                    )?,
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        23,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::modeling_type_facts::Kind::Indexed(
                wire,
            ) => {
                let alternative_path = child_path(path, "Indexed");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        24,
                    >>::VERSION,
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        24,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "element"), (2, "axes")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
                            '_,
                        >,
                    >(
                        required(
                            wire.element.as_option(),
                            &child_path(path, "element"),
                        )?,
                        &child_path(path, "element"),
                    )?,
                    {
                        let current_path = &child_path(path, "axes");
                        let values = wire
                            .axes
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::DeclarationIdView<
                                        '_,
                                    >,
                                >(value, &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::portable::TypeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        24,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
        'buffer,
    >,
> for pse_modeling::portable::TypeRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::portable::TypeRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::portable::TypeRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
                    '_,
                >,
            >(
                required(wire.value.as_option(), &child_path(path, "value"))?,
                &child_path(path, "value"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingUncertaintyKindView<
        'buffer,
    >,
> for pse_model::generated::enums::ModelingUncertaintyKind {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingUncertaintyKindView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValidityLayerView<
        'buffer,
    >,
> for pse_model::generated::enums::ModelingValidityLayer {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValidityLayerView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
        'buffer,
    >,
> for pse_modeling::portable::ValueRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::portable::ValueRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::portable::ValueRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingStaticValueFactsView<
                    '_,
                >,
            >(
                required(wire.value.as_option(), &child_path(path, "value"))?,
                &child_path(path, "value"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingVariableDomainView<
        'buffer,
    >,
> for pse_model::generated::enums::ModelingVariableDomain {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingVariableDomainView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::OpcodeView<
        'buffer,
    >,
> for pse_quantity::enums::Opcode {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::OpcodeView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::OperationIdView<
        'buffer,
    >,
> for pse_quantity::ids::OperationId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::OperationIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::OperationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::OperationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalBuiltInRuleView<
        'buffer,
    >,
> for pse_quantity::infer::BuiltInRule {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalBuiltInRuleView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalCanonicalConversionRecordView<
        'buffer,
    >,
> for pse_quantity::unit::CanonicalConversionRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalCanonicalConversionRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::unit::CanonicalConversionRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::CanonicalConversionRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "quantity"),
                (2, "context"),
                (3, "source"),
                (4, "target"),
                (5, "scale_bits"),
                (6, "offset_bits"),
                (7, "already_canonical"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                    '_,
                >,
            >(
                required(wire.quantity.as_option(), &child_path(path, "quantity"))?,
                &child_path(path, "quantity"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeKeyView<
                    '_,
                >,
            >(
                required(wire.context.as_option(), &child_path(path, "context"))?,
                &child_path(path, "context"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitIdView<
                    '_,
                >,
            >(
                required(wire.source.as_option(), &child_path(path, "source"))?,
                &child_path(path, "source"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitIdView<
                    '_,
                >,
            >(
                required(wire.target.as_option(), &child_path(path, "target"))?,
                &child_path(path, "target"),
            )?,
            native_from::<_, u64>(wire.scale_bits, &child_path(path, "scale_bits"))?,
            native_from::<_, u64>(wire.offset_bits, &child_path(path, "offset_bits"))?,
            native_from::<
                _,
                bool,
            >(wire.already_canonical, &child_path(path, "already_canonical"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalCanonicalMagnitudeRecordView<
        'buffer,
    >,
> for pse_quantity::unit::MagnitudeRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalCanonicalMagnitudeRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::unit::MagnitudeRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::MagnitudeRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "bits"), (2, "quantity")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.bits, &child_path(path, "bits"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                    '_,
                >,
            >(
                required(wire.quantity.as_option(), &child_path(path, "quantity"))?,
                &child_path(path, "quantity"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::ContractBoundaryRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractBoundaryRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::ContractBoundaryRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::ContractBoundaryRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "source"), (2, "target"), (3, "scale_bits")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                    '_,
                >,
            >(
                required(wire.source.as_option(), &child_path(path, "source"))?,
                &child_path(path, "source"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                    '_,
                >,
            >(
                required(wire.target.as_option(), &child_path(path, "target"))?,
                &child_path(path, "target"),
            )?,
            native_from::<_, u64>(wire.scale_bits, &child_path(path, "scale_bits"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractFactsView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::ContractWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::ContractWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::ContractWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "named"),
                (2, "kinds"),
                (3, "qualified"),
                (4, "axes"),
                (5, "indices"),
                (6, "factors"),
                (7, "dimension"),
                (8, "scale_bits"),
                (9, "pure_number"),
                (10, "formula"),
            ],
            path,
        )?;
        let fields = (
            wire
                .named
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                            '_,
                        >,
                    >(value, &child_path(path, "named"))
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "kinds");
                let values = wire
                    .kinds
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KindFactorView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "qualified");
                let values = wire
                    .qualified
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalQualifiedFactorView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "axes");
                let values = wire
                    .axes
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::EntityKindIdView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "indices");
                let values = wire
                    .indices
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BoundIndexReferenceView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "factors");
                let values = wire
                    .factors
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitFactorView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalDimensionVectorView<
                    '_,
                >,
            >(
                required(wire.dimension.as_option(), &child_path(path, "dimension"))?,
                &child_path(path, "dimension"),
            )?,
            native_from::<_, u64>(wire.scale_bits, &child_path(path, "scale_bits"))?,
            native_from::<_, bool>(wire.pure_number, &child_path(path, "pure_number"))?,
            wire
                .formula
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalFormulaFactsView<
                            '_,
                        >,
                    >(value, &child_path(path, "formula"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::ContractRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::ContractRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::ContractRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractFactsView<
                    '_,
                >,
            >(
                required(wire.value.as_option(), &child_path(path, "value"))?,
                &child_path(path, "value"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalDimensionVectorView<
        'buffer,
    >,
> for pse_quantity::dimension::DimensionVector {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalDimensionVectorView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Array8PhysicalRatioView<
                    '_,
                >,
            >(
                required(wire.exponents.as_option(), &child_path(path, "exponents"))?,
                &child_path(path, "exponents"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalFormulaFactsView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::FormulaWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalFormulaFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::FormulaWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::FormulaWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "authority"), (2, "operation"), (3, "operands"), (4, "powers")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.authority,
                &child_path(path, "authority"),
            )?,
            native_from::<_, &str>(wire.operation, &child_path(path, "operation"))?,
            {
                let current_path = &child_path(path, "operands");
                let values = wire
                    .operands
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractFactsView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "powers");
                let values = wire
                    .powers
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceFactsView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::InferenceWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::InferenceWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::InferenceWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "operands"),
                (2, "result"),
                (3, "selected"),
                (4, "conversions"),
                (5, "operand_scales"),
                (6, "result_scale"),
            ],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "operands");
                let values = wire
                    .operands
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractFactsView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractFactsView<
                    '_,
                >,
            >(
                required(wire.result.as_option(), &child_path(path, "result"))?,
                &child_path(path, "result"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOperationSelectionView<
                    '_,
                >,
            >(
                required(wire.selected.as_option(), &child_path(path, "selected"))?,
                &child_path(path, "selected"),
            )?,
            {
                let current_path = &child_path(path, "conversions");
                let values = wire
                    .conversions
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOperandConversionView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "operand_scales");
                let values = wire
                    .operand_scales
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            u64,
                        >(*(value), &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<_, u64>(wire.result_scale, &child_path(path, "result_scale"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceReceiptView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::Receipt {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceReceiptView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::Receipt as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::Receipt as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "request"), (2, "result")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.request,
                &child_path(path, "request"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceFactsView<
                    '_,
                >,
            >(
                required(wire.result.as_option(), &child_path(path, "result"))?,
                &child_path(path, "result"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::InferenceRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::InferenceRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::InferenceRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceFactsView<
                    '_,
                >,
            >(
                required(wire.value.as_option(), &child_path(path, "value"))?,
                &child_path(path, "value"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOperandConversionView<
        'buffer,
    >,
> for pse_quantity::infer::OperandConversion {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOperandConversionView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::infer::OperandConversion as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::infer::OperandConversion as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "operand"), (2, "conversion")],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.operand, &child_path(path, "operand"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ConversionIdView<
                    '_,
                >,
            >(
                required(wire.conversion.as_option(), &child_path(path, "conversion"))?,
                &child_path(path, "conversion"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOperationSelectionView<
        'buffer,
    >,
> for pse_quantity::infer::OperationSelection {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOperationSelectionView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::physical_operation_selection::Kind::BuiltIn(
                wire,
            ) => {
                let alternative_path = child_path(path, "BuiltIn");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::infer::OperationSelection as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_quantity::infer::OperationSelection as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "rule")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalBuiltInRuleView<
                            '_,
                        >,
                    >(
                        required(wire.rule.as_option(), &child_path(path, "rule"))?,
                        &child_path(path, "rule"),
                    )?,
                );
                Ok(
                    <pse_quantity::infer::OperationSelection as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::physical_operation_selection::Kind::Registered(
                wire,
            ) => {
                let alternative_path = child_path(path, "Registered");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::infer::OperationSelection as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_quantity::infer::OperationSelection as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "operation"), (2, "operand_permutation")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::OperationIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.operation.as_option(),
                            &child_path(path, "operation"),
                        )?,
                        &child_path(path, "operation"),
                    )?,
                    {
                        let current_path = &child_path(path, "operand_permutation");
                        let values = wire
                            .operand_permutation
                            .iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from::<
                                    _,
                                    u32,
                                >(*(value), &format!("{current_path}[{index}]"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_quantity::infer::OperationSelection as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryFactsView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::BoundaryWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::BoundaryWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::BoundaryWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "operation"), (2, "source"), (3, "target"), (4, "scale_bits")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.operation,
                &child_path(path, "operation"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractFactsView<
                    '_,
                >,
            >(
                required(wire.source.as_option(), &child_path(path, "source"))?,
                &child_path(path, "source"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractFactsView<
                    '_,
                >,
            >(
                required(wire.target.as_option(), &child_path(path, "target"))?,
                &child_path(path, "target"),
            )?,
            native_from::<_, u64>(wire.scale_bits, &child_path(path, "scale_bits"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryReceiptView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::BoundaryReceipt {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryReceiptView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::BoundaryReceipt as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::BoundaryReceipt as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "request"), (2, "result")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.request,
                &child_path(path, "request"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryFactsView<
                    '_,
                >,
            >(
                required(wire.result.as_option(), &child_path(path, "result"))?,
                &child_path(path, "result"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryRecordView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::OutputBoundaryRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::OutputBoundaryRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::OutputBoundaryRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "operation"), (2, "source"), (3, "target"), (4, "scale_bits")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.operation,
                &child_path(path, "operation"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                    '_,
                >,
            >(
                required(wire.source.as_option(), &child_path(path, "source"))?,
                &child_path(path, "source"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                    '_,
                >,
            >(
                required(wire.target.as_option(), &child_path(path, "target"))?,
                &child_path(path, "target"),
            )?,
            native_from::<_, u64>(wire.scale_bits, &child_path(path, "scale_bits"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalQualifiedFactorView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::QualifiedFactorWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalQualifiedFactorView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::QualifiedFactorWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::QualifiedFactorWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "key"), (2, "exponent")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeKeyView<
                    '_,
                >,
            >(
                required(wire.key.as_option(), &child_path(path, "key"))?,
                &child_path(path, "key"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
                    '_,
                >,
            >(
                required(wire.exponent.as_option(), &child_path(path, "exponent"))?,
                &child_path(path, "exponent"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
        'buffer,
    >,
> for pse_quantity::dimension::Ratio {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalReceiptStreamView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::ReceiptStream {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalReceiptStreamView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::ReceiptStream as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::ReceiptStream as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "version"), (2, "entries"), (3, "boundaries")],
            path,
        )?;
        let fields = (
            native_from::<_, u32>(wire.version, &child_path(path, "version"))?,
            {
                let current_path = &child_path(path, "entries");
                let values = wire
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalInferenceReceiptView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "boundaries");
                let values = wire
                    .boundaries
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalOutputBoundaryReceiptView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRepresentationConversionRecordView<
        'buffer,
    >,
> for pse_quantity::unit::RepresentationConversionRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRepresentationConversionRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::unit::RepresentationConversionRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit::RepresentationConversionRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "quantity"),
                (2, "context"),
                (3, "source"),
                (4, "target"),
                (5, "scale_bits"),
                (6, "offset_bits"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                    '_,
                >,
            >(
                required(wire.quantity.as_option(), &child_path(path, "quantity"))?,
                &child_path(path, "quantity"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeKeyView<
                    '_,
                >,
            >(
                required(wire.context.as_option(), &child_path(path, "context"))?,
                &child_path(path, "context"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitIdView<
                    '_,
                >,
            >(
                required(wire.source.as_option(), &child_path(path, "source"))?,
                &child_path(path, "source"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitIdView<
                    '_,
                >,
            >(
                required(wire.target.as_option(), &child_path(path, "target"))?,
                &child_path(path, "target"),
            )?,
            native_from::<_, u64>(wire.scale_bits, &child_path(path, "scale_bits"))?,
            native_from::<_, u64>(wire.offset_bits, &child_path(path, "offset_bits"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeRecordView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::SchemeRecord {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::SchemeRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::SchemeRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeShapeView<
                    '_,
                >,
            >(
                required(wire.value.as_option(), &child_path(path, "value"))?,
                &child_path(path, "value"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeShapeView<
        'buffer,
    >,
> for pse_quantity::resolved::receipts::SchemeWire {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeShapeView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        match wire
            .kind
            .as_ref()
            .ok_or_else(|| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "required alternative is absent",
            ))?
        {
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::physical_scheme_shape::Kind::Concrete(
                wire,
            ) => {
                let alternative_path = child_path(path, "Concrete");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "quantity")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                            '_,
                        >,
                    >(
                        required(
                            wire.quantity.as_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::physical_scheme_shape::Kind::Resolved(
                wire,
            ) => {
                let alternative_path = child_path(path, "Resolved");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "contract")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                            '_,
                        >,
                    >(
                        required(
                            wire.contract.as_option(),
                            &child_path(path, "contract"),
                        )?,
                        &child_path(path, "contract"),
                    )?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::physical_scheme_shape::Kind::Variable(
                wire,
            ) => {
                let alternative_path = child_path(path, "Variable");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "name")],
                    path,
                )?;
                let fields = (
                    native_from::<_, &str>(wire.name, &child_path(path, "name"))?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::physical_scheme_shape::Kind::Delta(
                wire,
            ) => {
                let alternative_path = child_path(path, "Delta");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeShapeView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::physical_scheme_shape::Kind::Product(
                wire,
            ) => {
                let alternative_path = child_path(path, "Product");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "left"), (2, "right")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeShapeView<
                            '_,
                        >,
                    >(
                        required(wire.left.as_option(), &child_path(path, "left"))?,
                        &child_path(path, "left"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeShapeView<
                            '_,
                        >,
                    >(
                        required(wire.right.as_option(), &child_path(path, "right"))?,
                        &child_path(path, "right"),
                    )?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::physical_scheme_shape::Kind::Quotient(
                wire,
            ) => {
                let alternative_path = child_path(path, "Quotient");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "left"), (2, "right")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeShapeView<
                            '_,
                        >,
                    >(
                        required(wire.left.as_option(), &child_path(path, "left"))?,
                        &child_path(path, "left"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeShapeView<
                            '_,
                        >,
                    >(
                        required(wire.right.as_option(), &child_path(path, "right"))?,
                        &child_path(path, "right"),
                    )?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::oneof::physical_scheme_shape::Kind::Power(
                wire,
            ) => {
                let alternative_path = child_path(path, "Power");
                let path = alternative_path.as_str();
                unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value"), (2, "power")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalSchemeShapeView<
                            '_,
                        >,
                    >(
                        required(wire.value.as_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
                            '_,
                        >,
                    >(
                        required(wire.power.as_option(), &child_path(path, "power"))?,
                        &child_path(path, "power"),
                    )?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalUnitProductView<
        'buffer,
    >,
> for pse_quantity::unit_product::UnitProduct {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalUnitProductView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::unit_product::UnitProduct as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::unit_product::UnitProduct as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "factors")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "factors");
                let values = wire
                    .factors
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfStringAndPhysicalRatioView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityKindIdView<
        'buffer,
    >,
> for pse_quantity::ids::QuantityKindId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityKindIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::QuantityKindId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityKindId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
        'buffer,
    >,
> for pse_quantity::ids::QuantityTypeId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::QuantityTypeId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::QuantityTypeId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeKeyView<
        'buffer,
    >,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeKeyView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityKindIdView<
                    '_,
                >,
            >(
                required(wire.kind.as_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            wire
                .basis
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::BasisIdView<
                            '_,
                        >,
                    >(value, &child_path(path, "basis"))
                })
                .transpose()?,
            wire
                .reference_state
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ReferenceStateIdView<
                            '_,
                        >,
                    >(value, &child_path(path, "reference_state"))
                })
                .transpose()?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ScaleKindView<
                    '_,
                >,
            >(
                required(wire.scale_kind.as_option(), &child_path(path, "scale_kind"))?,
                &child_path(path, "scale_kind"),
            )?,
            {
                let current_path = &child_path(path, "shape");
                let values = wire
                    .shape
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::EntityKindIdView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .subject_kind
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::EntityKindIdView<
                            '_,
                        >,
                    >(value, &child_path(path, "subject_kind"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ReductionKindView<
        'buffer,
    >,
> for pse_quantity::enums::ReductionKind {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ReductionKindView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ReferenceStateIdView<
        'buffer,
    >,
> for pse_quantity::ids::ReferenceStateId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ReferenceStateIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::ReferenceStateId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::ReferenceStateId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ScaleKindView<
        'buffer,
    >,
> for pse_quantity::enums::ScaleKind {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ScaleKindView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        wire.value
            .parse::<Self>()
            .map_err(|_| refused(
                path,
                AdapterFailure::UnknownAlternative,
                "unknown canonical source vocabulary member",
            ))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ScientificRecipeView<
        'buffer,
    >,
> for crate::typed_math::portable::Recipe {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ScientificRecipeView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::typed_math::portable::Recipe as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::typed_math::portable::Recipe as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "interpretation"),
                (2, "semantic_identity"),
                (3, "syntax"),
                (4, "support"),
                (5, "providers"),
                (6, "inputs"),
                (7, "physical"),
                (8, "structure"),
                (9, "slots"),
                (10, "occurrences"),
                (11, "outputs"),
                (12, "checked_members"),
                (13, "checked_sources"),
                (14, "occurrence_tokens"),
                (15, "receipts"),
                (16, "proofs"),
                (17, "spec"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &str,
            >(wire.interpretation, &child_path(path, "interpretation"))?,
            wire
                .semantic_identity
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::SemanticBodyHashView<
                            '_,
                        >,
                    >(value, &child_path(path, "semantic_identity"))
                })
                .transpose()?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedRecordView<
                    '_,
                >,
            >(
                required(wire.syntax.as_option(), &child_path(path, "syntax"))?,
                &child_path(path, "syntax"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingSupportRecordView<
                    '_,
                >,
            >(
                required(wire.support.as_option(), &child_path(path, "support"))?,
                &child_path(path, "support"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::CompilerSelectedProvidersRecordView<
                    '_,
                >,
            >(
                required(wire.providers.as_option(), &child_path(path, "providers"))?,
                &child_path(path, "providers"),
            )?,
            {
                let current_path = &child_path(path, "inputs");
                let values = wire
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSymbolIxView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.physical,
                &child_path(path, "physical"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.structure,
                &child_path(path, "structure"),
            )?,
            native_from::<_, u64>(wire.slots, &child_path(path, "slots"))?,
            native_from::<_, u64>(wire.occurrences, &child_path(path, "occurrences"))?,
            {
                let current_path = &child_path(path, "outputs");
                let values = wire
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::QuantityTypeIdView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "checked_members");
                let values = wire
                    .checked_members
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "checked_sources");
                let values = wire
                    .checked_sources
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "occurrence_tokens");
                let values = wire
                    .occurrence_tokens
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                            value,
                            &format!("{current_path}[{index}]"),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalReceiptStreamView<
                    '_,
                >,
            >(
                required(wire.receipts.as_option(), &child_path(path, "receipts"))?,
                &child_path(path, "receipts"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathematicalProofStreamView<
                    '_,
                >,
            >(
                required(wire.proofs.as_option(), &child_path(path, "proofs"))?,
                &child_path(path, "proofs"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::MathBodySpecView<
                    '_,
                >,
            >(
                required(wire.spec.as_option(), &child_path(path, "spec"))?,
                &child_path(path, "spec"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::SemanticBodyHashView<
        'buffer,
    >,
> for pse_ids::roles::SemanticBodyHash {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::SemanticBodyHashView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_ids::roles::SemanticBodyHash as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_ids::roles::SemanticBodyHash as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfContentHashAndKernelProviderRecordView<
        'buffer,
    >,
> for (pse_ids::ContentHash, pse_kernels::portable::ProviderRecord) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfContentHashAndKernelProviderRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                pse_ids::ContentHash,
                pse_kernels::portable::ProviderRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                pse_ids::ContentHash,
                pse_kernels::portable::ProviderRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.first,
                &child_path(path, "first"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::KernelProviderRecordView<
                    '_,
                >,
            >(
                required(wire.second.as_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfListModelingValueRecordAndOptionalModelingResolvedSymbolIxView<
        'buffer,
    >,
>
for (
    Vec<pse_modeling::portable::ValueRecord>,
    Option<pse_modeling::resolved::SymbolIx>,
) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfListModelingValueRecordAndOptionalModelingResolvedSymbolIxView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                Vec<pse_modeling::portable::ValueRecord>,
                Option<pse_modeling::resolved::SymbolIx>,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                Vec<pse_modeling::portable::ValueRecord>,
                Option<pse_modeling::resolved::SymbolIx>,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            {
                let current_path = &child_path(path, "first");
                let values = wire
                    .first
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .second
                .as_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSymbolIxView<
                            '_,
                        >,
                    >(value, &child_path(path, "second"))
                })
                .transpose()?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingResolvedNodeIxAndModelingPhysicalAdmissionRecordView<
        'buffer,
    >,
>
for (
    pse_modeling::resolved::NodeIx,
    pse_modeling::expression::admission::PhysicalAdmissionRecord,
) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingResolvedNodeIxAndModelingPhysicalAdmissionRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                pse_modeling::resolved::NodeIx,
                pse_modeling::expression::admission::PhysicalAdmissionRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                pse_modeling::resolved::NodeIx,
                pse_modeling::expression::admission::PhysicalAdmissionRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                    '_,
                >,
            >(
                required(wire.first.as_option(), &child_path(path, "first"))?,
                &child_path(path, "first"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingPhysicalAdmissionRecordView<
                    '_,
                >,
            >(
                required(wire.second.as_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingResolvedNodeIxAndModelingTypeRecordView<
        'buffer,
    >,
> for (pse_modeling::resolved::NodeIx, pse_modeling::portable::TypeRecord) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingResolvedNodeIxAndModelingTypeRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                pse_modeling::resolved::NodeIx,
                pse_modeling::portable::TypeRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                pse_modeling::resolved::NodeIx,
                pse_modeling::portable::TypeRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedNodeIxView<
                    '_,
                >,
            >(
                required(wire.first.as_option(), &child_path(path, "first"))?,
                &child_path(path, "first"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeRecordView<
                    '_,
                >,
            >(
                required(wire.second.as_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingResolvedSymbolIxAndModelingMemberRangeView<
        'buffer,
    >,
> for (pse_modeling::resolved::SymbolIx, pse_modeling::resolved::MemberRange) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingResolvedSymbolIxAndModelingMemberRangeView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                pse_modeling::resolved::SymbolIx,
                pse_modeling::resolved::MemberRange,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                pse_modeling::resolved::SymbolIx,
                pse_modeling::resolved::MemberRange,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedSymbolIxView<
                    '_,
                >,
            >(
                required(wire.first.as_option(), &child_path(path, "first"))?,
                &child_path(path, "first"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingMemberRangeView<
                    '_,
                >,
            >(
                required(wire.second.as_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingValueRecordAndListModelingValueRecordView<
        'buffer,
    >,
> for (pse_modeling::portable::ValueRecord, Vec<pse_modeling::portable::ValueRecord>) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfModelingValueRecordAndListModelingValueRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                pse_modeling::portable::ValueRecord,
                Vec<pse_modeling::portable::ValueRecord>,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                pse_modeling::portable::ValueRecord,
                Vec<pse_modeling::portable::ValueRecord>,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                    '_,
                >,
            >(
                required(wire.first.as_option(), &child_path(path, "first"))?,
                &child_path(path, "first"),
            )?,
            {
                let current_path = &child_path(path, "second");
                let values = wire
                    .second
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfStringAndModelingTypeFactsView<
        'buffer,
    >,
> for (String, pse_modeling::portable::TypeWire) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfStringAndModelingTypeFactsView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                String,
                pse_modeling::portable::TypeWire,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                String,
                pse_modeling::portable::TypeWire,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<_, &str>(wire.first, &child_path(path, "first"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingTypeFactsView<
                    '_,
                >,
            >(
                required(wire.second.as_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfStringAndModelingValueRecordView<
        'buffer,
    >,
> for (String, pse_modeling::portable::ValueRecord) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfStringAndModelingValueRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                String,
                pse_modeling::portable::ValueRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                String,
                pse_modeling::portable::ValueRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<_, &str>(wire.first, &child_path(path, "first"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                    '_,
                >,
            >(
                required(wire.second.as_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfStringAndPhysicalRatioView<
        'buffer,
    >,
> for (String, pse_quantity::dimension::Ratio) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfStringAndPhysicalRatioView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                String,
                pse_quantity::dimension::Ratio,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                String,
                pse_quantity::dimension::Ratio,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<_, &str>(wire.first, &child_path(path, "first"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
                    '_,
                >,
            >(
                required(wire.second.as_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfUsizeAndPhysicalContractRecordView<
        'buffer,
    >,
> for (usize, pse_quantity::resolved::receipts::ContractRecord) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple2OfUsizeAndPhysicalContractRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                usize,
                pse_quantity::resolved::receipts::ContractRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                usize,
                pse_quantity::resolved::receipts::ContractRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.first, &child_path(path, "first"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalContractRecordView<
                    '_,
                >,
            >(
                required(wire.second.as_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple3OfStringAndModelingUncertaintyKindAndU64View<
        'buffer,
    >,
> for (String, pse_model::generated::enums::ModelingUncertaintyKind, u64) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple3OfStringAndModelingUncertaintyKindAndU64View<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                String,
                pse_model::generated::enums::ModelingUncertaintyKind,
                u64,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                String,
                pse_model::generated::enums::ModelingUncertaintyKind,
                u64,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second"), (3, "third")],
            path,
        )?;
        let fields = (
            native_from::<_, &str>(wire.first, &child_path(path, "first"))?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingUncertaintyKindView<
                    '_,
                >,
            >(
                required(wire.second.as_option(), &child_path(path, "second"))?,
                &child_path(path, "second"),
            )?,
            native_from::<_, u64>(wire.third, &child_path(path, "third"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple3OfStringAndUsizeAndSemanticIdView<
        'buffer,
    >,
> for (String, usize, pse_ids::SemanticId) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple3OfStringAndUsizeAndSemanticIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
            native_from::<_, &str>(wire.first, &child_path(path, "first"))?,
            native_from::<_, u64>(wire.second, &child_path(path, "second"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                wire.third,
                &child_path(path, "third"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple3OfUsizeAndListModelingValueRecordAndModelingResolvedCarrierRecordView<
        'buffer,
    >,
>
for (
    usize,
    Vec<pse_modeling::portable::ValueRecord>,
    pse_modeling::resolved::records::CarrierRecord,
) {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::Tuple3OfUsizeAndListModelingValueRecordAndModelingResolvedCarrierRecordView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                usize,
                Vec<pse_modeling::portable::ValueRecord>,
                pse_modeling::resolved::records::CarrierRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                usize,
                Vec<pse_modeling::portable::ValueRecord>,
                pse_modeling::resolved::records::CarrierRecord,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second"), (3, "third")],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.first, &child_path(path, "first"))?,
            {
                let current_path = &child_path(path, "second");
                let values = wire
                    .second
                    .iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from::<
                            _,
                            &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingValueRecordView<
                                '_,
                            >,
                        >(value, &format!("{current_path}[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ModelingResolvedCarrierRecordView<
                    '_,
                >,
            >(
                required(wire.third.as_option(), &child_path(path, "third"))?,
                &child_path(path, "third"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitFactorView<
        'buffer,
    >,
> for pse_quantity::unit::UnitFactor {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitFactorView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
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
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitIdView<
                    '_,
                >,
            >(
                required(wire.unit.as_option(), &child_path(path, "unit"))?,
                &child_path(path, "unit"),
            )?,
            native_from::<
                _,
                &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::PhysicalRatioView<
                    '_,
                >,
            >(
                required(wire.exponent.as_option(), &child_path(path, "exponent"))?,
                &child_path(path, "exponent"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
impl<
    'buffer,
> NativeFrom<
    &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitIdView<
        'buffer,
    >,
> for pse_quantity::ids::UnitId {
    fn from_wire(
        wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::UnitIdView<
            'buffer,
        >,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields_view(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::UnitId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::UnitId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_slice(
                required(wire.id.as_ref(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|source| source.at(path))
    }
}
/// Validate and adopt the selected native `ScientificRecipe` directly from its eager view.
pub(super) fn decode_scientific_recipe_view(
    wire: &crate::typed_math::portable::generated::pb::pse::scientific_recipe::v4::__buffa::view::ScientificRecipeView<
        '_,
    >,
) -> Result<crate::typed_math::portable::Recipe, BoundaryProjectionError> {
    decode_view_root(wire, "ScientificRecipe")
}
