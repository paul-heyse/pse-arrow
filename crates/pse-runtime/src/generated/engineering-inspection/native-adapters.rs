use super::{
    AdapterError, AdapterFailure, BoundaryProjectionError, NativeInto, NativeFrom,
    layout, child_path, refused, required, unknown_fields, native_into, native_from,
    native_into_in_scope, native_from_in_scope, encode_root_in_scope,
    decode_root_in_scope, items, from_items, unique_pairs, wire_map,
};
impl NativeInto<
    crate::host_operations::engineering::inspection::types::Array8PhysicalRatio,
    crate::host_operations::engineering::NativeAdapterScope,
> for [pse_quantity::dimension::Ratio; 8] {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::Array8PhysicalRatio,
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
        Ok(crate::host_operations::engineering::inspection::types::Array8PhysicalRatio {
            values: {
                let current_path = &child_path(path, "values");
                items(field_0)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::PhysicalRatio,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::Array8PhysicalRatio,
    crate::host_operations::engineering::NativeAdapterScope,
> for [pse_quantity::dimension::Ratio; 8] {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::Array8PhysicalRatio,
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
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::PhysicalRatio,
                            crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::BasisId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::BasisId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::BasisId,
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
        Ok(crate::host_operations::engineering::inspection::types::BasisId {
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
    crate::host_operations::engineering::inspection::types::BasisId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::BasisId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::BasisId,
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
    crate::host_operations::engineering::inspection::types::BoundIndexId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::BoundIndexId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::BoundIndexId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::BoundIndexId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::BoundIndexId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::BoundIndexId {
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
    crate::host_operations::engineering::inspection::types::BoundIndexId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::BoundIndexId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::BoundIndexId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::BoundIndexId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::BoundIndexId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
    crate::host_operations::engineering::inspection::types::BoundIndexReference,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::index::BoundIndexRef {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::BoundIndexReference,
        AdapterError,
    > {
        layout(
            <pse_quantity::index::BoundIndexRef as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::index::BoundIndexRef as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "bound_index"), (2, "domain"), (3, "kind")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::BoundIndexReference {
            bound_index: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::BoundIndexId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "bound_index"))?,
            ),
            domain: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::DomainId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "domain"))?,
            ),
            kind: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::EntityKindId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "kind"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::BoundIndexReference,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::index::BoundIndexRef {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::BoundIndexReference,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::index::BoundIndexRef as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::index::BoundIndexRef as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "bound_index"), (2, "domain"), (3, "kind")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::BoundIndexId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.bound_index.into_option(),
                    &child_path(path, "bound_index"),
                )?,
                &child_path(path, "bound_index"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::DomainId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.domain.into_option(), &child_path(path, "domain"))?,
                &child_path(path, "domain"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::EntityKindId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::CanonicalContextCut,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_operations::canonical_contexts::ContextCut {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::CanonicalContextCut,
        AdapterError,
    > {
        layout(
            <pse_operations::canonical_contexts::ContextCut as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_contexts::ContextCut as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "issuing_store"), (2, "project"), (3, "sequence")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::CanonicalContextCut {
            issuing_store: native_into::<
                _,
                String,
            >(field_0, &child_path(path, "issuing_store"))?,
            project: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::CanonicalProjectKey,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "project"))?,
            ),
            sequence: native_into::<_, u64>(field_2, &child_path(path, "sequence"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::CanonicalContextCut,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_operations::canonical_contexts::ContextCut {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::CanonicalContextCut,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_operations::canonical_contexts::ContextCut as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_contexts::ContextCut as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "issuing_store"), (2, "project"), (3, "sequence")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                String,
            >(wire.issuing_store, &child_path(path, "issuing_store"))?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::CanonicalProjectKey,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.project.into_option(), &child_path(path, "project"))?,
                &child_path(path, "project"),
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
    crate::host_operations::engineering::inspection::types::CanonicalContextSelection,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_operations::canonical_contexts::ContextSelection {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::CanonicalContextSelection,
        AdapterError,
    > {
        layout(
            <pse_operations::canonical_contexts::ContextSelection as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_contexts::ContextSelection as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "cut"), (2, "effective_at_us"), (3, "context")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::CanonicalContextSelection {
            cut: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::CanonicalContextCut,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "cut"))?,
            ),
            effective_at_us: native_into::<
                _,
                i64,
            >(field_1, &child_path(path, "effective_at_us"))?,
            context: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::CanonicalProjectContext,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "context"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::CanonicalContextSelection,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_operations::canonical_contexts::ContextSelection {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::CanonicalContextSelection,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_operations::canonical_contexts::ContextSelection as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_operations::canonical_contexts::ContextSelection as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "cut"), (2, "effective_at_us"), (3, "context")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::CanonicalContextCut,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.cut.into_option(), &child_path(path, "cut"))?,
                &child_path(path, "cut"),
            )?,
            native_from::<
                _,
                i64,
            >(wire.effective_at_us, &child_path(path, "effective_at_us"))?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::CanonicalProjectContext,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.context.into_option(), &child_path(path, "context"))?,
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
    crate::host_operations::engineering::inspection::types::CanonicalProjectContext,
    crate::host_operations::engineering::NativeAdapterScope,
>
for pse_model::generated::runtime::canonical_project_contexts::RuntimeCanonicalProjectContextsRow {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::CanonicalProjectContext,
        AdapterError,
    > {
        layout(
            <pse_model::generated::runtime::canonical_project_contexts::RuntimeCanonicalProjectContextsRow as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::runtime::canonical_project_contexts::RuntimeCanonicalProjectContextsRow as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "key"),
                (2, "project"),
                (3, "scope"),
                (4, "revision"),
                (5, "source_identity"),
                (6, "physical_context"),
                (7, "physical_revision"),
                (8, "physical_source_identity"),
                (9, "effective_from_us"),
                (10, "effective_to_us"),
                (11, "recorded_sequence"),
                (12, "corrects"),
                (13, "deleted"),
                (14, "request"),
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
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::CanonicalProjectContext {
            key: native_into::<_, String>(field_0, &child_path(path, "key"))?,
            project: native_into::<_, String>(field_1, &child_path(path, "project"))?,
            scope: native_into::<_, String>(field_2, &child_path(path, "scope"))?,
            revision: native_into::<_, String>(field_3, &child_path(path, "revision"))?,
            source_identity: native_into::<
                _,
                String,
            >(field_4, &child_path(path, "source_identity"))?,
            physical_context: native_into::<
                _,
                String,
            >(field_5, &child_path(path, "physical_context"))?,
            physical_revision: native_into::<
                _,
                String,
            >(field_6, &child_path(path, "physical_revision"))?,
            physical_source_identity: native_into::<
                _,
                String,
            >(field_7, &child_path(path, "physical_source_identity"))?,
            effective_from_us: native_into::<
                _,
                i64,
            >(field_8, &child_path(path, "effective_from_us"))?,
            effective_to_us: field_9
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, i64>(value, &child_path(path, "effective_to_us"))
                })
                .transpose()?,
            recorded_sequence: native_into::<
                _,
                u64,
            >(field_10, &child_path(path, "recorded_sequence"))?,
            corrects: field_11
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, String>(value, &child_path(path, "corrects"))
                })
                .transpose()?,
            deleted: native_into::<_, bool>(field_12, &child_path(path, "deleted"))?,
            request: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_13,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::CanonicalProjectContext,
    crate::host_operations::engineering::NativeAdapterScope,
>
for pse_model::generated::runtime::canonical_project_contexts::RuntimeCanonicalProjectContextsRow {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::CanonicalProjectContext,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::generated::runtime::canonical_project_contexts::RuntimeCanonicalProjectContextsRow as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::runtime::canonical_project_contexts::RuntimeCanonicalProjectContextsRow as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "key"),
                (2, "project"),
                (3, "scope"),
                (4, "revision"),
                (5, "source_identity"),
                (6, "physical_context"),
                (7, "physical_revision"),
                (8, "physical_source_identity"),
                (9, "effective_from_us"),
                (10, "effective_to_us"),
                (11, "recorded_sequence"),
                (12, "corrects"),
                (13, "deleted"),
                (14, "request"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.key, &child_path(path, "key"))?,
            native_from::<_, String>(wire.project, &child_path(path, "project"))?,
            native_from::<_, String>(wire.scope, &child_path(path, "scope"))?,
            native_from::<_, String>(wire.revision, &child_path(path, "revision"))?,
            native_from::<
                _,
                String,
            >(wire.source_identity, &child_path(path, "source_identity"))?,
            native_from::<
                _,
                String,
            >(wire.physical_context, &child_path(path, "physical_context"))?,
            native_from::<
                _,
                String,
            >(wire.physical_revision, &child_path(path, "physical_revision"))?,
            native_from::<
                _,
                String,
            >(
                wire.physical_source_identity,
                &child_path(path, "physical_source_identity"),
            )?,
            native_from::<
                _,
                i64,
            >(wire.effective_from_us, &child_path(path, "effective_from_us"))?,
            wire
                .effective_to_us
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, i64>(value, &child_path(path, "effective_to_us"))
                })
                .transpose()?,
            native_from::<
                _,
                u64,
            >(wire.recorded_sequence, &child_path(path, "recorded_sequence"))?,
            wire
                .corrects
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, String>(value, &child_path(path, "corrects"))
                })
                .transpose()?,
            native_from::<_, bool>(wire.deleted, &child_path(path, "deleted"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.request,
                &child_path(path, "request"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::CanonicalProjectKey,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::canonical::ProjectKey {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::CanonicalProjectKey,
        AdapterError,
    > {
        layout(
            <pse_model::generated::canonical::ProjectKey as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::canonical::ProjectKey as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::CanonicalProjectKey {
            value: native_into::<_, String>(field_0, &child_path(path, "value"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::CanonicalProjectKey,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::canonical::ProjectKey {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::CanonicalProjectKey,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::generated::canonical::ProjectKey as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::canonical::ProjectKey as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
    crate::host_operations::engineering::inspection::types::CompilerProfile,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_compiler::workspace::Profile {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::CompilerProfile,
        AdapterError,
    > {
        layout(
            <pse_compiler::workspace::Profile as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_compiler::workspace::Profile as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "optimization"),
                (2, "evaluation"),
                (3, "assembly"),
                (4, "class_proof_work"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::CompilerProfile {
            optimization: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::MathOptimization,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "optimization"))?,
            ),
            evaluation: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::MathEvaluationLimits,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "evaluation"))?,
            ),
            assembly: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::MathAssemblyLimits,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "assembly"))?,
            ),
            class_proof_work: native_into::<
                _,
                u64,
            >(field_3, &child_path(path, "class_proof_work"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::CompilerProfile,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_compiler::workspace::Profile {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::CompilerProfile,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_compiler::workspace::Profile as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_compiler::workspace::Profile as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "optimization"),
                (2, "evaluation"),
                (3, "assembly"),
                (4, "class_proof_work"),
            ],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::MathOptimization,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.optimization.into_option(),
                    &child_path(path, "optimization"),
                )?,
                &child_path(path, "optimization"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::MathEvaluationLimits,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.evaluation.into_option(),
                    &child_path(path, "evaluation"),
                )?,
                &child_path(path, "evaluation"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::MathAssemblyLimits,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.assembly.into_option(), &child_path(path, "assembly"))?,
                &child_path(path, "assembly"),
            )?,
            native_from::<
                _,
                u64,
            >(wire.class_proof_work, &child_path(path, "class_proof_work"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::DeclarationId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::identities::DeclarationId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::DeclarationId,
        AdapterError,
    > {
        layout(
            <pse_model::generated::identities::DeclarationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::DeclarationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::DeclarationId {
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
    crate::host_operations::engineering::inspection::types::DeclarationId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::identities::DeclarationId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::DeclarationId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::generated::identities::DeclarationId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::DeclarationId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
    crate::host_operations::engineering::inspection::types::DomainId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::DomainId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::DomainId,
        AdapterError,
    > {
        layout(
            <pse_quantity::ids::DomainId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::DomainId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::DomainId {
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
    crate::host_operations::engineering::inspection::types::DomainId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::DomainId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::DomainId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::ids::DomainId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::ids::DomainId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
    crate::host_operations::engineering::inspection::types::EntityKindId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::EntityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::EntityKindId,
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
        Ok(crate::host_operations::engineering::inspection::types::EntityKindId {
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
    crate::host_operations::engineering::inspection::types::EntityKindId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::EntityKindId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::EntityKindId,
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
    crate::host_operations::engineering::inspection::types::InspectionConnection,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionConnection {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionConnection,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionConnection as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionConnection as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "from"), (3, "to"), (4, "lineage")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionConnection {
            id: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            from: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            to: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_2,
            ),
            lineage: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::InspectionLineage,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_3, &child_path(path, "lineage"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionConnection,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionConnection {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionConnection,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionConnection as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionConnection as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "from"), (3, "to"), (4, "lineage")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.id,
                &child_path(path, "id"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.from,
                &child_path(path, "from"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.to,
                &child_path(path, "to"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::InspectionLineage,
                crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::InspectionDomain,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionDomain {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionDomain,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionDomain as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionDomain as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "key"), (2, "carrier")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionDomain {
            key: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingDomainKey,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "key"))?,
            ),
            carrier: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingCarrier,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "carrier"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionDomain,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionDomain {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionDomain,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionDomain as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionDomain as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "key"), (2, "carrier")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingDomainKey,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.key.into_option(), &child_path(path, "key"))?,
                &child_path(path, "key"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingCarrier,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.carrier.into_option(), &child_path(path, "carrier"))?,
                &child_path(path, "carrier"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::InspectionExecution,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionExecution {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionExecution,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionExecution as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionExecution as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "route"), (2, "procedure"), (3, "requested_start")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionExecution {
            route: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingAnalysisRoute,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "route"))?,
            ),
            procedure: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingProcedure,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "procedure"))?,
            ),
            requested_start: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::NativeStartPolicy,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "requested_start"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionExecution,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionExecution {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionExecution,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionExecution as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionExecution as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "route"), (2, "procedure"), (3, "requested_start")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingAnalysisRoute,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.route.into_option(), &child_path(path, "route"))?,
                &child_path(path, "route"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingProcedure,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.procedure.into_option(), &child_path(path, "procedure"))?,
                &child_path(path, "procedure"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::NativeStartPolicy,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.requested_start.into_option(),
                    &child_path(path, "requested_start"),
                )?,
                &child_path(path, "requested_start"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::InspectionFixture,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionFixture {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionFixture,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionFixture as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionFixture as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "declaration"),
                (2, "specifications"),
                (3, "integration"),
                (4, "initial_equations"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionFixture {
            declaration: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::DeclarationId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "declaration"))?,
            ),
            specifications: {
                let current_path = &child_path(path, "specifications");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionSpecification,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            integration: field_2
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::InspectionIntegration,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "integration"))
                })
                .transpose()?
                .into(),
            initial_equations: items(field_3)
                .map(
                    pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes,
                )
                .collect::<Vec<_>>(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionFixture,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionFixture {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionFixture,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionFixture as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionFixture as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "declaration"),
                (2, "specifications"),
                (3, "integration"),
                (4, "initial_equations"),
            ],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::DeclarationId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.declaration.into_option(),
                    &child_path(path, "declaration"),
                )?,
                &child_path(path, "declaration"),
            )?,
            {
                let current_path = &child_path(path, "specifications");
                let values = wire
                    .specifications
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionSpecification,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .integration
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::InspectionIntegration,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "integration"))
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "initial_equations");
                let values = wire
                    .initial_equations
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
    crate::host_operations::engineering::inspection::types::InspectionInstance,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionInstance {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionInstance,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionInstance as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionInstance as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "definition"), (3, "parent"), (4, "path"), (5, "members")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionInstance {
            id: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::InstanceId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "id"))?,
            ),
            definition: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::DeclarationId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "definition"))?,
            ),
            parent: field_2
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::InstanceId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "parent"))
                })
                .transpose()?
                .into(),
            path: native_into::<_, String>(field_3, &child_path(path, "path"))?,
            members: wire_map(
                field_4
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_into::<_, String>(key, &child_path(path, "members"))?,
                            native_into_in_scope::<
                                _,
                                crate::host_operations::engineering::inspection::types::DeclarationId,
                                crate::host_operations::engineering::NativeAdapterScope,
                            >(value, &child_path(path, "members"))?,
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
    crate::host_operations::engineering::inspection::types::InspectionInstance,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionInstance {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionInstance,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionInstance as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionInstance as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "definition"), (3, "parent"), (4, "path"), (5, "members")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::InstanceId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.id.into_option(), &child_path(path, "id"))?,
                &child_path(path, "id"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::DeclarationId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.definition.into_option(),
                    &child_path(path, "definition"),
                )?,
                &child_path(path, "definition"),
            )?,
            wire
                .parent
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::InstanceId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "parent"))
                })
                .transpose()?,
            native_from::<_, String>(wire.path, &child_path(path, "path"))?,
            unique_pairs(
                wire
                    .members
                    .into_iter()
                    .map(|(key, value)| -> Result<_, AdapterError> {
                        Ok((
                            native_from::<_, String>(key, &child_path(path, "members"))?,
                            native_from_in_scope::<
                                _,
                                crate::host_operations::engineering::inspection::types::DeclarationId,
                                crate::host_operations::engineering::NativeAdapterScope,
                            >(value, &child_path(path, "members"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                &child_path(path, "members"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::InspectionIntegration,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionIntegration {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionIntegration,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionIntegration as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionIntegration as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "axis"),
                (2, "quantity_id"),
                (3, "unit_id"),
                (4, "lower"),
                (5, "upper"),
                (6, "initial_step"),
                (7, "relative_tolerance"),
                (8, "normalized_absolute_tolerance"),
                (9, "quadrature_relative_tolerance"),
                (10, "samples"),
                (11, "quadratures"),
                (12, "schedule_intervals"),
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
        Ok(crate::host_operations::engineering::inspection::types::InspectionIntegration {
            axis: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            quantity_id: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::QuantityTypeId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "quantity_id"))?,
            ),
            unit_id: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::UnitId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "unit_id"))?,
            ),
            lower: native_into::<_, u64>(field_3, &child_path(path, "lower"))?,
            upper: native_into::<_, u64>(field_4, &child_path(path, "upper"))?,
            initial_step: native_into::<
                _,
                u64,
            >(field_5, &child_path(path, "initial_step"))?,
            relative_tolerance: native_into::<
                _,
                u64,
            >(field_6, &child_path(path, "relative_tolerance"))?,
            normalized_absolute_tolerance: native_into::<
                _,
                u64,
            >(field_7, &child_path(path, "normalized_absolute_tolerance"))?,
            quadrature_relative_tolerance: field_8
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<
                        _,
                        u64,
                    >(value, &child_path(path, "quadrature_relative_tolerance"))
                })
                .transpose()?,
            samples: {
                let current_path = &child_path(path, "samples");
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
            quadratures: {
                let current_path = &child_path(path, "quadratures");
                items(field_10)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionQuadrature,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            schedule_intervals: {
                let current_path = &child_path(path, "schedule_intervals");
                items(field_11)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionScheduleInterval,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionIntegration,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionIntegration {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionIntegration,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionIntegration as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionIntegration as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "axis"),
                (2, "quantity_id"),
                (3, "unit_id"),
                (4, "lower"),
                (5, "upper"),
                (6, "initial_step"),
                (7, "relative_tolerance"),
                (8, "normalized_absolute_tolerance"),
                (9, "quadrature_relative_tolerance"),
                (10, "samples"),
                (11, "quadratures"),
                (12, "schedule_intervals"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.axis,
                &child_path(path, "axis"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::QuantityTypeId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.quantity_id.into_option(),
                    &child_path(path, "quantity_id"),
                )?,
                &child_path(path, "quantity_id"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::UnitId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.unit_id.into_option(), &child_path(path, "unit_id"))?,
                &child_path(path, "unit_id"),
            )?,
            native_from::<_, u64>(wire.lower, &child_path(path, "lower"))?,
            native_from::<_, u64>(wire.upper, &child_path(path, "upper"))?,
            native_from::<_, u64>(wire.initial_step, &child_path(path, "initial_step"))?,
            native_from::<
                _,
                u64,
            >(wire.relative_tolerance, &child_path(path, "relative_tolerance"))?,
            native_from::<
                _,
                u64,
            >(
                wire.normalized_absolute_tolerance,
                &child_path(path, "normalized_absolute_tolerance"),
            )?,
            wire
                .quadrature_relative_tolerance
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<
                        _,
                        u64,
                    >(value, &child_path(path, "quadrature_relative_tolerance"))
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "samples");
                let values = wire
                    .samples
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
                let current_path = &child_path(path, "quadratures");
                let values = wire
                    .quadratures
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionQuadrature,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "schedule_intervals");
                let values = wire
                    .schedule_intervals
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionScheduleInterval,
                            crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::InspectionLineage,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionLineage {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionLineage,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionLineage as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionLineage as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "declaration"),
                (2, "instance"),
                (3, "path"),
                (4, "demand"),
                (5, "default_owner"),
                (6, "is_override"),
                (7, "presets"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionLineage {
            declaration: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::DeclarationId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "declaration"))?,
            ),
            instance: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::InstanceId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "instance"))?,
            ),
            path: native_into::<_, String>(field_2, &child_path(path, "path"))?,
            demand: {
                let current_path = &child_path(path, "demand");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::DeclarationId,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            default_owner: field_4
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "default_owner"))
                })
                .transpose()?
                .into(),
            is_override: native_into::<
                _,
                bool,
            >(field_5, &child_path(path, "is_override"))?,
            presets: {
                let current_path = &child_path(path, "presets");
                items(field_6)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::DeclarationId,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionLineage,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionLineage {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionLineage,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionLineage as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionLineage as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "declaration"),
                (2, "instance"),
                (3, "path"),
                (4, "demand"),
                (5, "default_owner"),
                (6, "is_override"),
                (7, "presets"),
            ],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::DeclarationId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.declaration.into_option(),
                    &child_path(path, "declaration"),
                )?,
                &child_path(path, "declaration"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::InstanceId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.instance.into_option(), &child_path(path, "instance"))?,
                &child_path(path, "instance"),
            )?,
            native_from::<_, String>(wire.path, &child_path(path, "path"))?,
            {
                let current_path = &child_path(path, "demand");
                let values = wire
                    .demand
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::DeclarationId,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .default_owner
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "default_owner"))
                })
                .transpose()?,
            native_from::<_, bool>(wire.is_override, &child_path(path, "is_override"))?,
            {
                let current_path = &child_path(path, "presets");
                let values = wire
                    .presets
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::DeclarationId,
                            crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::InspectionMember,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionMember {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionMember,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionMember as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionMember as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "role"), (3, "lineage"), (4, "ty")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionMember {
            id: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            role: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingDeclarationKind,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "role"))?,
            ),
            lineage: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::InspectionLineage,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "lineage"))?,
            ),
            ty: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingType,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_3, &child_path(path, "ty"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionMember,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionMember {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionMember,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionMember as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionMember as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "role"), (3, "lineage"), (4, "ty")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.id,
                &child_path(path, "id"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingDeclarationKind,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.role.into_option(), &child_path(path, "role"))?,
                &child_path(path, "role"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::InspectionLineage,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.lineage.into_option(), &child_path(path, "lineage"))?,
                &child_path(path, "lineage"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingType,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.ty.into_option(), &child_path(path, "ty"))?,
                &child_path(path, "ty"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::InspectionPopulation,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionPopulation {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionPopulation,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionPopulation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionPopulation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "instances"),
                (2, "members"),
                (3, "ports"),
                (4, "connections"),
                (5, "domains"),
                (6, "specifications"),
                (7, "integration_samples"),
                (8, "integration_quadratures"),
                (9, "schedule_intervals"),
                (10, "initial_equations"),
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
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionPopulation {
            instances: native_into::<_, u64>(field_0, &child_path(path, "instances"))?,
            members: native_into::<_, u64>(field_1, &child_path(path, "members"))?,
            ports: native_into::<_, u64>(field_2, &child_path(path, "ports"))?,
            connections: native_into::<
                _,
                u64,
            >(field_3, &child_path(path, "connections"))?,
            domains: native_into::<_, u64>(field_4, &child_path(path, "domains"))?,
            specifications: native_into::<
                _,
                u64,
            >(field_5, &child_path(path, "specifications"))?,
            integration_samples: native_into::<
                _,
                u64,
            >(field_6, &child_path(path, "integration_samples"))?,
            integration_quadratures: native_into::<
                _,
                u64,
            >(field_7, &child_path(path, "integration_quadratures"))?,
            schedule_intervals: native_into::<
                _,
                u64,
            >(field_8, &child_path(path, "schedule_intervals"))?,
            initial_equations: native_into::<
                _,
                u64,
            >(field_9, &child_path(path, "initial_equations"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionPopulation,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionPopulation {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionPopulation,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionPopulation as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionPopulation as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "instances"),
                (2, "members"),
                (3, "ports"),
                (4, "connections"),
                (5, "domains"),
                (6, "specifications"),
                (7, "integration_samples"),
                (8, "integration_quadratures"),
                (9, "schedule_intervals"),
                (10, "initial_equations"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.instances, &child_path(path, "instances"))?,
            native_from::<_, u64>(wire.members, &child_path(path, "members"))?,
            native_from::<_, u64>(wire.ports, &child_path(path, "ports"))?,
            native_from::<_, u64>(wire.connections, &child_path(path, "connections"))?,
            native_from::<_, u64>(wire.domains, &child_path(path, "domains"))?,
            native_from::<
                _,
                u64,
            >(wire.specifications, &child_path(path, "specifications"))?,
            native_from::<
                _,
                u64,
            >(wire.integration_samples, &child_path(path, "integration_samples"))?,
            native_from::<
                _,
                u64,
            >(
                wire.integration_quadratures,
                &child_path(path, "integration_quadratures"),
            )?,
            native_from::<
                _,
                u64,
            >(wire.schedule_intervals, &child_path(path, "schedule_intervals"))?,
            native_from::<
                _,
                u64,
            >(wire.initial_equations, &child_path(path, "initial_equations"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::InspectionPort,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionPort {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionPort,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionPort as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionPort as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "symbol"), (3, "lineage")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionPort {
            id: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            symbol: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            lineage: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::InspectionLineage,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "lineage"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionPort,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionPort {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionPort,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionPort as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionPort as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "symbol"), (3, "lineage")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.id,
                &child_path(path, "id"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.symbol,
                &child_path(path, "symbol"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::InspectionLineage,
                crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::InspectionQuadrature,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionQuadrature {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionQuadrature,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionQuadrature as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionQuadrature as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "target"),
                (2, "quantity_id"),
                (3, "unit_id"),
                (4, "absolute_tolerance"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionQuadrature {
            target: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            quantity_id: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::QuantityTypeId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "quantity_id"))?,
            ),
            unit_id: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::UnitId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "unit_id"))?,
            ),
            absolute_tolerance: native_into::<
                _,
                u64,
            >(field_3, &child_path(path, "absolute_tolerance"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionQuadrature,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionQuadrature {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionQuadrature,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionQuadrature as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionQuadrature as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "target"),
                (2, "quantity_id"),
                (3, "unit_id"),
                (4, "absolute_tolerance"),
            ],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.target,
                &child_path(path, "target"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::QuantityTypeId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.quantity_id.into_option(),
                    &child_path(path, "quantity_id"),
                )?,
                &child_path(path, "quantity_id"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::UnitId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.unit_id.into_option(), &child_path(path, "unit_id"))?,
                &child_path(path, "unit_id"),
            )?,
            native_from::<
                _,
                u64,
            >(wire.absolute_tolerance, &child_path(path, "absolute_tolerance"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::InspectionScheduleControl,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionScheduleControl {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionScheduleControl,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionScheduleControl as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionScheduleControl as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "lower"), (2, "upper")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionScheduleControl {
            lower: field_0
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "lower"))
                })
                .transpose()?,
            upper: field_1
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "upper"))
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionScheduleControl,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionScheduleControl {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionScheduleControl,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionScheduleControl as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionScheduleControl as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "lower"), (2, "upper")],
            path,
        )?;
        let fields = (
            wire
                .lower
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u64>(value, &child_path(path, "lower"))
                })
                .transpose()?,
            wire
                .upper
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u64>(value, &child_path(path, "upper"))
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
    crate::host_operations::engineering::inspection::types::InspectionScheduleInterval,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionScheduleInterval {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionScheduleInterval,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionScheduleInterval as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionScheduleInterval as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "schedule"),
                (2, "target"),
                (3, "quantity_id"),
                (4, "unit_id"),
                (5, "interval"),
                (6, "change_time"),
                (7, "value"),
                (8, "control"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InspectionScheduleInterval {
            schedule: native_into::<_, u64>(field_0, &child_path(path, "schedule"))?,
            target: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            quantity_id: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::QuantityTypeId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "quantity_id"))?,
            ),
            unit_id: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::UnitId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_3, &child_path(path, "unit_id"))?,
            ),
            interval: native_into::<_, u64>(field_4, &child_path(path, "interval"))?,
            change_time: field_5
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "change_time"))
                })
                .transpose()?,
            value: native_into::<_, u64>(field_6, &child_path(path, "value"))?,
            control: field_7
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::InspectionScheduleControl,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "control"))
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionScheduleInterval,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionScheduleInterval {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionScheduleInterval,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionScheduleInterval as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionScheduleInterval as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "schedule"),
                (2, "target"),
                (3, "quantity_id"),
                (4, "unit_id"),
                (5, "interval"),
                (6, "change_time"),
                (7, "value"),
                (8, "control"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.schedule, &child_path(path, "schedule"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.target,
                &child_path(path, "target"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::QuantityTypeId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.quantity_id.into_option(),
                    &child_path(path, "quantity_id"),
                )?,
                &child_path(path, "quantity_id"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::UnitId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.unit_id.into_option(), &child_path(path, "unit_id"))?,
                &child_path(path, "unit_id"),
            )?,
            native_from::<_, u64>(wire.interval, &child_path(path, "interval"))?,
            wire
                .change_time
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u64>(value, &child_path(path, "change_time"))
                })
                .transpose()?,
            native_from::<_, u64>(wire.value, &child_path(path, "value"))?,
            wire
                .control
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::InspectionScheduleControl,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "control"))
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
    crate::host_operations::engineering::inspection::types::InspectionSpecification,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionSpecification {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InspectionSpecification,
        AdapterError,
    > {
        layout(
            <crate::workflow::InspectionSpecification as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionSpecification as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "path"),
                (2, "target"),
                (3, "ty"),
                (4, "quantity_id"),
                (5, "unit_id"),
                (6, "value"),
                (7, "fixed"),
                (8, "lower"),
                (9, "upper"),
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
        Ok(crate::host_operations::engineering::inspection::types::InspectionSpecification {
            path: native_into::<_, String>(field_0, &child_path(path, "path"))?,
            target: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_1,
            ),
            ty: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingType,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "ty"))?,
            ),
            quantity_id: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::QuantityTypeId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_3, &child_path(path, "quantity_id"))?,
            ),
            unit_id: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::UnitId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_4, &child_path(path, "unit_id"))?,
            ),
            value: field_5
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "value"))
                })
                .transpose()?,
            fixed: field_6
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, bool>(value, &child_path(path, "fixed"))
                })
                .transpose()?,
            lower: field_7
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "lower"))
                })
                .transpose()?,
            upper: field_8
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "upper"))
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::InspectionSpecification,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::InspectionSpecification {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InspectionSpecification,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::InspectionSpecification as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::InspectionSpecification as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "path"),
                (2, "target"),
                (3, "ty"),
                (4, "quantity_id"),
                (5, "unit_id"),
                (6, "value"),
                (7, "fixed"),
                (8, "lower"),
                (9, "upper"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.path, &child_path(path, "path"))?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.target,
                &child_path(path, "target"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingType,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.ty.into_option(), &child_path(path, "ty"))?,
                &child_path(path, "ty"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::QuantityTypeId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.quantity_id.into_option(),
                    &child_path(path, "quantity_id"),
                )?,
                &child_path(path, "quantity_id"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::UnitId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.unit_id.into_option(), &child_path(path, "unit_id"))?,
                &child_path(path, "unit_id"),
            )?,
            wire
                .value
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u64>(value, &child_path(path, "value"))
                })
                .transpose()?,
            wire
                .fixed
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, bool>(value, &child_path(path, "fixed"))
                })
                .transpose()?,
            wire
                .lower
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u64>(value, &child_path(path, "lower"))
                })
                .transpose()?,
            wire
                .upper
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u64>(value, &child_path(path, "upper"))
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
    crate::host_operations::engineering::inspection::types::InstanceId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::identities::InstanceId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::InstanceId,
        AdapterError,
    > {
        layout(
            <pse_model::generated::identities::InstanceId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::InstanceId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::InstanceId {
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
    crate::host_operations::engineering::inspection::types::InstanceId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::identities::InstanceId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::InstanceId,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_model::generated::identities::InstanceId as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_model::generated::identities::InstanceId as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
    crate::host_operations::engineering::inspection::types::KindFactor,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::kind::KindFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::KindFactor,
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
        Ok(crate::host_operations::engineering::inspection::types::KindFactor {
            kind: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::QuantityKindId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "kind"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::PhysicalRatio,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::KindFactor,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::kind::KindFactor {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::KindFactor,
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
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::QuantityKindId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::PhysicalRatio,
                crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::MathAssemblyLimits,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_math::assembly::AssemblyLimits {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::MathAssemblyLimits,
        AdapterError,
    > {
        layout(
            <pse_math::assembly::AssemblyLimits as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::assembly::AssemblyLimits as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "contributions"), (2, "native_index"), (3, "worker_bytes")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::MathAssemblyLimits {
            contributions: native_into::<
                _,
                u64,
            >(field_0, &child_path(path, "contributions"))?,
            native_index: native_into::<
                _,
                u64,
            >(field_1, &child_path(path, "native_index"))?,
            worker_bytes: native_into::<
                _,
                u64,
            >(field_2, &child_path(path, "worker_bytes"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::MathAssemblyLimits,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_math::assembly::AssemblyLimits {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::MathAssemblyLimits,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::assembly::AssemblyLimits as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::assembly::AssemblyLimits as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "contributions"), (2, "native_index"), (3, "worker_bytes")],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                u64,
            >(wire.contributions, &child_path(path, "contributions"))?,
            native_from::<_, u64>(wire.native_index, &child_path(path, "native_index"))?,
            native_from::<_, u64>(wire.worker_bytes, &child_path(path, "worker_bytes"))?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::MathEvaluationLimits,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_math::jets::EvaluationLimits {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::MathEvaluationLimits,
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
        Ok(crate::host_operations::engineering::inspection::types::MathEvaluationLimits {
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
    crate::host_operations::engineering::inspection::types::MathEvaluationLimits,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_math::jets::EvaluationLimits {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::MathEvaluationLimits,
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
    crate::host_operations::engineering::inspection::types::MathEvaluator,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_math::library::Evaluator {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::MathEvaluator,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::MathEvaluator {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::math_evaluator::Kind::Interpreted(
                            Box::new(crate::host_operations::engineering::inspection::types::math_evaluator::Alternative1 {
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
            <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "module_bytes")],
            path,
        )?;
        let value = match <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "CompiledScalar");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::MathEvaluator {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::math_evaluator::Kind::CompiledScalar(
                            Box::new(crate::host_operations::engineering::inspection::types::math_evaluator::Alternative2 {
                                module_bytes: native_into::<
                                    _,
                                    u64,
                                >(field_0, &child_path(path, "module_bytes"))?,
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
    crate::host_operations::engineering::inspection::types::MathEvaluator,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_math::library::Evaluator {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::MathEvaluator,
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::math_evaluator::Kind::Interpreted(
                wire,
            ) => {
                let alternative_path = child_path(path, "Interpreted");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::math_evaluator::Kind::CompiledScalar(
                wire,
            ) => {
                let alternative_path = child_path(path, "CompiledScalar");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "module_bytes")],
                    path,
                )?;
                let fields = (
                    native_from::<
                        _,
                        u64,
                    >(wire.module_bytes, &child_path(path, "module_bytes"))?,
                );
                Ok(
                    <pse_math::library::Evaluator as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::MathOptimization,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_math::library::Optimization {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::MathOptimization,
        AdapterError,
    > {
        layout(
            <pse_math::library::Optimization as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::library::Optimization as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "cores"),
                (2, "horner_iterations"),
                (3, "cpe_iterations"),
                (4, "evaluator"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::MathOptimization {
            cores: native_into::<_, u64>(field_0, &child_path(path, "cores"))?,
            horner_iterations: native_into::<
                _,
                u64,
            >(field_1, &child_path(path, "horner_iterations"))?,
            cpe_iterations: native_into::<
                _,
                u64,
            >(field_2, &child_path(path, "cpe_iterations"))?,
            evaluator: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::MathEvaluator,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_3, &child_path(path, "evaluator"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::MathOptimization,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_math::library::Optimization {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::MathOptimization,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_math::library::Optimization as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_math::library::Optimization as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "cores"),
                (2, "horner_iterations"),
                (3, "cpe_iterations"),
                (4, "evaluator"),
            ],
            path,
        )?;
        let fields = (
            native_from::<_, u64>(wire.cores, &child_path(path, "cores"))?,
            native_from::<
                _,
                u64,
            >(wire.horner_iterations, &child_path(path, "horner_iterations"))?,
            native_from::<
                _,
                u64,
            >(wire.cpe_iterations, &child_path(path, "cpe_iterations"))?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::MathEvaluator,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.evaluator.into_option(), &child_path(path, "evaluator"))?,
                &child_path(path, "evaluator"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::ModelingAnalysisRoute,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::enums::ModelingAnalysisRoute {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingAnalysisRoute,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::inspection::types::ModelingAnalysisRoute {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ModelingAnalysisRoute,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::enums::ModelingAnalysisRoute {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingAnalysisRoute,
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
    crate::host_operations::engineering::inspection::types::ModelingBoundaryRef,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::BoundaryRef {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingBoundaryRef,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Declared");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingBoundaryRef {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_boundary_ref::Kind::Declared(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_boundary_ref::Alternative1 {
                                value: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "instance"), (2, "declaration"), (3, "coordinates")],
            path,
        )?;
        let value = match <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1, field_2)) => {
                let alternative_path = child_path(path, "Bound");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingBoundaryRef {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_boundary_ref::Kind::Bound(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_boundary_ref::Alternative2 {
                                instance: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::InstanceId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "instance"))?,
                                ),
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "declaration"))?,
                                ),
                                coordinates: {
                                    let current_path = &child_path(path, "coordinates");
                                    items(field_2)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into_in_scope::<
                                                _,
                                                crate::host_operations::engineering::inspection::types::StructuralValue,
                                                crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::ModelingBoundaryRef,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::BoundaryRef {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingBoundaryRef,
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_boundary_ref::Kind::Declared(
                wire,
            ) => {
                let alternative_path = child_path(path, "Declared");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_boundary_ref::Kind::Bound(
                wire,
            ) => {
                let alternative_path = child_path(path, "Bound");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "instance"), (2, "declaration"), (3, "coordinates")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::InstanceId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.instance.into_option(),
                            &child_path(path, "instance"),
                        )?,
                        &child_path(path, "instance"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                    {
                        let current_path = &child_path(path, "coordinates");
                        let values = wire
                            .coordinates
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from_in_scope::<
                                    _,
                                    crate::host_operations::engineering::inspection::types::StructuralValue,
                                    crate::host_operations::engineering::NativeAdapterScope,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::BoundaryRef as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::ModelingCarrier,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::resolved::Carrier {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingCarrier,
        AdapterError,
    > {
        layout(
            <pse_modeling::resolved::Carrier as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::Carrier as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "kind"), (2, "element"), (3, "members")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::ModelingCarrier {
            kind: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingCarrierKind,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "kind"))?,
            ),
            element: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingType,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "element"))?,
            ),
            members: {
                let current_path = &child_path(path, "members");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::StructuralValue,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ModelingCarrier,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::resolved::Carrier {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingCarrier,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::resolved::Carrier as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::resolved::Carrier as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "kind"), (2, "element"), (3, "members")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingCarrierKind,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingType,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.element.into_option(), &child_path(path, "element"))?,
                &child_path(path, "element"),
            )?,
            {
                let current_path = &child_path(path, "members");
                let values = wire
                    .members
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::StructuralValue,
                            crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::ModelingCarrierKind,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::resolved::CarrierKind {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingCarrierKind,
        AdapterError,
    > {
        let value = self;
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
        let value = match <pse_modeling::resolved::CarrierKind as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::ModelingCarrierKind {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_carrier_kind::Kind::OrderedSet(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_carrier_kind::Alternative1 {
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
        let value = match <pse_modeling::resolved::CarrierKind as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Continuous");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingCarrierKind {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_carrier_kind::Kind::Continuous(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_carrier_kind::Alternative2 {
                                domain: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "domain"))?,
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
    crate::host_operations::engineering::inspection::types::ModelingCarrierKind,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::resolved::CarrierKind {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingCarrierKind,
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_carrier_kind::Kind::OrderedSet(
                wire,
            ) => {
                let alternative_path = child_path(path, "OrderedSet");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_carrier_kind::Kind::Continuous(
                wire,
            ) => {
                let alternative_path = child_path(path, "Continuous");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.domain.into_option(),
                            &child_path(path, "domain"),
                        )?,
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
impl NativeInto<
    crate::host_operations::engineering::inspection::types::ModelingDeclarationKind,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::enums::ModelingDeclarationKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingDeclarationKind,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::inspection::types::ModelingDeclarationKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ModelingDeclarationKind,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::enums::ModelingDeclarationKind {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingDeclarationKind,
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
    crate::host_operations::engineering::inspection::types::ModelingDomainKey,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::specialize::finite::DomainKey {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingDomainKey,
        AdapterError,
    > {
        layout(
            <pse_modeling::specialize::finite::DomainKey as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::finite::DomainKey as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "family"), (2, "position"), (3, "prefix")],
            path,
        )?;
        let (field_0, field_1, field_2) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::ModelingDomainKey {
            family: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingFamilyKey,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "family"))?,
            ),
            position: native_into::<_, u64>(field_1, &child_path(path, "position"))?,
            prefix: {
                let current_path = &child_path(path, "prefix");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::StructuralValue,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ModelingDomainKey,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::specialize::finite::DomainKey {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingDomainKey,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::specialize::finite::DomainKey as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::finite::DomainKey as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "family"), (2, "position"), (3, "prefix")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingFamilyKey,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.family.into_option(), &child_path(path, "family"))?,
                &child_path(path, "family"),
            )?,
            native_from::<_, u64>(wire.position, &child_path(path, "position"))?,
            {
                let current_path = &child_path(path, "prefix");
                let values = wire
                    .prefix
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::StructuralValue,
                            crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::ModelingFamilyKey,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::specialize::finite::FamilyKey {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingFamilyKey,
        AdapterError,
    > {
        layout(
            <pse_modeling::specialize::finite::FamilyKey as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::finite::FamilyKey as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "instance"), (2, "declaration"), (3, "role"), (4, "parents")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::ModelingFamilyKey {
            instance: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::InstanceId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "instance"))?,
            ),
            declaration: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::DeclarationId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "declaration"))?,
            ),
            role: native_into::<_, String>(field_2, &child_path(path, "role"))?,
            parents: {
                let current_path = &child_path(path, "parents");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::StructuralValue,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ModelingFamilyKey,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::specialize::finite::FamilyKey {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingFamilyKey,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::specialize::finite::FamilyKey as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::finite::FamilyKey as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "instance"), (2, "declaration"), (3, "role"), (4, "parents")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::InstanceId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.instance.into_option(), &child_path(path, "instance"))?,
                &child_path(path, "instance"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::DeclarationId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.declaration.into_option(),
                    &child_path(path, "declaration"),
                )?,
                &child_path(path, "declaration"),
            )?,
            native_from::<_, String>(wire.role, &child_path(path, "role"))?,
            {
                let current_path = &child_path(path, "parents");
                let values = wire
                    .parents
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::StructuralValue,
                            crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::ModelingInspection,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::ModelingInspection {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingInspection,
        AdapterError,
    > {
        layout(
            <crate::workflow::ModelingInspection as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::ModelingInspection as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "source"),
                (2, "preparation"),
                (3, "population"),
                (4, "offset"),
                (5, "next_offset"),
                (6, "execution"),
                (7, "instances"),
                (8, "members"),
                (9, "ports"),
                (10, "connections"),
                (11, "domains"),
                (12, "selected_root"),
                (13, "selected_instance"),
                (14, "fixture"),
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
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::ModelingInspection {
            source: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::OperationSource,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "source"))?,
            ),
            preparation: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::PreparationSettings,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "preparation"))?,
            ),
            population: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::InspectionPopulation,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "population"))?,
            ),
            offset: native_into::<_, u64>(field_3, &child_path(path, "offset"))?,
            next_offset: field_4
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "next_offset"))
                })
                .transpose()?,
            execution: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::InspectionExecution,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_5, &child_path(path, "execution"))?,
            ),
            instances: {
                let current_path = &child_path(path, "instances");
                items(field_6)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionInstance,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            members: {
                let current_path = &child_path(path, "members");
                items(field_7)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionMember,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ports: {
                let current_path = &child_path(path, "ports");
                items(field_8)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionPort,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            connections: {
                let current_path = &child_path(path, "connections");
                items(field_9)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionConnection,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            domains: {
                let current_path = &child_path(path, "domains");
                items(field_10)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionDomain,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            selected_root: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::DeclarationId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_11, &child_path(path, "selected_root"))?,
            ),
            selected_instance: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::InstanceId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_12, &child_path(path, "selected_instance"))?,
            ),
            fixture: field_13
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::InspectionFixture,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "fixture"))
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ModelingInspection,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::ModelingInspection {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingInspection,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::ModelingInspection as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::ModelingInspection as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "source"),
                (2, "preparation"),
                (3, "population"),
                (4, "offset"),
                (5, "next_offset"),
                (6, "execution"),
                (7, "instances"),
                (8, "members"),
                (9, "ports"),
                (10, "connections"),
                (11, "domains"),
                (12, "selected_root"),
                (13, "selected_instance"),
                (14, "fixture"),
            ],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::OperationSource,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.source.into_option(), &child_path(path, "source"))?,
                &child_path(path, "source"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::PreparationSettings,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.preparation.into_option(),
                    &child_path(path, "preparation"),
                )?,
                &child_path(path, "preparation"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::InspectionPopulation,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.population.into_option(),
                    &child_path(path, "population"),
                )?,
                &child_path(path, "population"),
            )?,
            native_from::<_, u64>(wire.offset, &child_path(path, "offset"))?,
            wire
                .next_offset
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u64>(value, &child_path(path, "next_offset"))
                })
                .transpose()?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::InspectionExecution,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.execution.into_option(), &child_path(path, "execution"))?,
                &child_path(path, "execution"),
            )?,
            {
                let current_path = &child_path(path, "instances");
                let values = wire
                    .instances
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionInstance,
                            crate::host_operations::engineering::NativeAdapterScope,
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
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionMember,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "ports");
                let values = wire
                    .ports
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionPort,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "connections");
                let values = wire
                    .connections
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionConnection,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "domains");
                let values = wire
                    .domains
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::InspectionDomain,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::DeclarationId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.selected_root.into_option(),
                    &child_path(path, "selected_root"),
                )?,
                &child_path(path, "selected_root"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::InstanceId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(
                    wire.selected_instance.into_option(),
                    &child_path(path, "selected_instance"),
                )?,
                &child_path(path, "selected_instance"),
            )?,
            wire
                .fixture
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::InspectionFixture,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "fixture"))
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
    crate::host_operations::engineering::inspection::types::ModelingLimits,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::specialize::Limits {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingLimits,
        AdapterError,
    > {
        layout(
            <pse_modeling::specialize::Limits as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::Limits as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
        let (field_0, field_1, field_2, field_3, field_4) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::ModelingLimits {
            depth: native_into::<_, u64>(field_0, &child_path(path, "depth"))?,
            items: native_into::<_, u64>(field_1, &child_path(path, "items"))?,
            members: native_into::<_, u64>(field_2, &child_path(path, "members"))?,
            body_occurrences: field_3
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "body_occurrences"))
                })
                .transpose()?,
            body_slots: field_4
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, u64>(value, &child_path(path, "body_slots"))
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ModelingLimits,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::specialize::Limits {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingLimits,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_modeling::specialize::Limits as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_modeling::specialize::Limits as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
                    native_from::<_, u64>(value, &child_path(path, "body_occurrences"))
                })
                .transpose()?,
            wire
                .body_slots
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, u64>(value, &child_path(path, "body_slots"))
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
    crate::host_operations::engineering::inspection::types::ModelingPhysicalRefinement,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::PhysicalRefinement {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingPhysicalRefinement,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "map"), (2, "slot")],
            path,
        )?;
        let value = match <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingPhysicalRefinement {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_physical_refinement::Kind::Coordinate(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_physical_refinement::Alternative1 {
                                map: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "map"))?,
                                ),
                                slot: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "slot"))?,
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
            <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "map"), (2, "reconstruction")],
            path,
        )?;
        let value = match <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "ReducedLaw");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingPhysicalRefinement {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_physical_refinement::Kind::ReducedLaw(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_physical_refinement::Alternative2 {
                                map: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "map"))?,
                                ),
                                reconstruction: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "reconstruction"))?,
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
            <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "boundary"), (2, "direction")],
            path,
        )?;
        let value = match <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Transfer");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingPhysicalRefinement {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_physical_refinement::Kind::Transfer(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_physical_refinement::Alternative3 {
                                boundary: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::ModelingBoundaryRef,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "boundary"))?,
                                ),
                                direction: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::ModelingTransferDirection,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "direction"))?,
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
    crate::host_operations::engineering::inspection::types::ModelingPhysicalRefinement,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::PhysicalRefinement {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingPhysicalRefinement,
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_physical_refinement::Kind::Coordinate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "map"), (2, "slot")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.map.into_option(), &child_path(path, "map"))?,
                        &child_path(path, "map"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.slot.into_option(), &child_path(path, "slot"))?,
                        &child_path(path, "slot"),
                    )?,
                );
                Ok(
                    <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_physical_refinement::Kind::ReducedLaw(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReducedLaw");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "map"), (2, "reconstruction")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.map.into_option(), &child_path(path, "map"))?,
                        &child_path(path, "map"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.reconstruction.into_option(),
                            &child_path(path, "reconstruction"),
                        )?,
                        &child_path(path, "reconstruction"),
                    )?,
                );
                Ok(
                    <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_physical_refinement::Kind::Transfer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Transfer");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "boundary"), (2, "direction")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ModelingBoundaryRef,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.boundary.into_option(),
                            &child_path(path, "boundary"),
                        )?,
                        &child_path(path, "boundary"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ModelingTransferDirection,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.direction.into_option(),
                            &child_path(path, "direction"),
                        )?,
                        &child_path(path, "direction"),
                    )?,
                );
                Ok(
                    <pse_modeling::PhysicalRefinement as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::ModelingProcedure,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::enums::ModelingProcedure {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingProcedure,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::inspection::types::ModelingProcedure {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ModelingProcedure,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::enums::ModelingProcedure {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingProcedure,
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
    crate::host_operations::engineering::inspection::types::ModelingTransferDirection,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::TransferDirection {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingTransferDirection,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::ModelingTransferDirection {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_transfer_direction::Kind::Into(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_transfer_direction::Alternative1 {
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
            <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::ModelingTransferDirection {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_transfer_direction::Kind::OutOf(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_transfer_direction::Alternative2 {
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
    crate::host_operations::engineering::inspection::types::ModelingTransferDirection,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::TransferDirection {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingTransferDirection,
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_transfer_direction::Kind::Into(
                wire,
            ) => {
                let alternative_path = child_path(path, "Into");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_transfer_direction::Kind::OutOf(
                wire,
            ) => {
                let alternative_path = child_path(path, "OutOf");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::TransferDirection as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::ModelingType,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::Type {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ModelingType,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Boolean(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative1 {
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Integer(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative2 {
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Text(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative3 {
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::QuantityType(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative4 {
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::ReferenceState(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative5 {
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Applicability(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative6 {
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            7,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Boundary");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Boundary(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative7 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            8,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "CoordinateMap");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::CoordinateMap(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative8 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                9,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                9,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration"), (2, "map")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            9,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Reconstruction");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Reconstruction(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative9 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
                                ),
                                map: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "map"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                10,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                10,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "scheme")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            10,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Quantity");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Quantity(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative10 {
                                scheme: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::QuantityScheme,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "scheme"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                11,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                11,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "quantity"), (2, "refinement")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            11,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "RefinedQuantity");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::RefinedQuantity(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative11 {
                                quantity: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::QuantityScheme,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "quantity"))?,
                                ),
                                refinement: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::ModelingPhysicalRefinement,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "refinement"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                12,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                12,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            12,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Entity");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Entity(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative12 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                13,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                13,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            13,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Enum");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Enum(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative13 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                14,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                14,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            14,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Identifier");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Identifier(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative14 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                15,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                15,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "element")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            15,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Set");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Set(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative15 {
                                element: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::ModelingType,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "element"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                16,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                16,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "domain"), (2, "element")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            16,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Continuous");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Continuous(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative16 {
                                domain: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "domain"))?,
                                ),
                                element: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::ModelingType,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "element"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                17,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                17,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "elements")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            17,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Tuple");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Tuple(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative17 {
                                elements: {
                                    let current_path = &child_path(path, "elements");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into_in_scope::<
                                                _,
                                                crate::host_operations::engineering::inspection::types::ModelingType,
                                                crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                18,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                18,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            18,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Table");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Table(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative18 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                19,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                19,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            19,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Row");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Row(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative19 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                20,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                20,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            20,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Definition");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Definition(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative20 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                21,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                21,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "declaration")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            21,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Interface");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Interface(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative21 {
                                declaration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "declaration"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                22,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                22,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "arguments"), (2, "result")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            22,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Function");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Function(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative22 {
                                arguments: {
                                    let current_path = &child_path(path, "arguments");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into_in_scope::<
                                                _,
                                                crate::host_operations::engineering::inspection::types::Tuple2OfStringAndModelingType,
                                                crate::host_operations::engineering::NativeAdapterScope,
                                            >(value, &format!("{path}[{index}]", path = current_path))
                                        })
                                        .collect::<Result<Vec<_>, _>>()?
                                },
                                result: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::ModelingType,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "result"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                23,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                23,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "element")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            23,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Optional");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Optional(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative23 {
                                element: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::ModelingType,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "element"))?,
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
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                24,
            >>::VERSION,
            <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                24,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "element"), (2, "axes")],
            path,
        )?;
        let value = match <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
            24,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Indexed");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::ModelingType {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Indexed(
                            Box::new(crate::host_operations::engineering::inspection::types::modeling_type::Alternative24 {
                                element: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::ModelingType,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "element"))?,
                                ),
                                axes: {
                                    let current_path = &child_path(path, "axes");
                                    items(field_1)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into_in_scope::<
                                                _,
                                                crate::host_operations::engineering::inspection::types::DeclarationId,
                                                crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::ModelingType,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::Type {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ModelingType,
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Boolean(
                wire,
            ) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Integer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Text(
                wire,
            ) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::QuantityType(
                wire,
            ) => {
                let alternative_path = child_path(path, "QuantityType");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::ReferenceState(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReferenceState");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Applicability(
                wire,
            ) => {
                let alternative_path = child_path(path, "Applicability");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Boundary(
                wire,
            ) => {
                let alternative_path = child_path(path, "Boundary");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::CoordinateMap(
                wire,
            ) => {
                let alternative_path = child_path(path, "CoordinateMap");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Reconstruction(
                wire,
            ) => {
                let alternative_path = child_path(path, "Reconstruction");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration"), (2, "map")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.map.into_option(), &child_path(path, "map"))?,
                        &child_path(path, "map"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Quantity(
                wire,
            ) => {
                let alternative_path = child_path(path, "Quantity");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "scheme")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::QuantityScheme,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.scheme.into_option(),
                            &child_path(path, "scheme"),
                        )?,
                        &child_path(path, "scheme"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::RefinedQuantity(
                wire,
            ) => {
                let alternative_path = child_path(path, "RefinedQuantity");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "quantity"), (2, "refinement")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::QuantityScheme,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.quantity.into_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ModelingPhysicalRefinement,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.refinement.into_option(),
                            &child_path(path, "refinement"),
                        )?,
                        &child_path(path, "refinement"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Entity(
                wire,
            ) => {
                let alternative_path = child_path(path, "Entity");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Enum(
                wire,
            ) => {
                let alternative_path = child_path(path, "Enum");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Identifier(
                wire,
            ) => {
                let alternative_path = child_path(path, "Identifier");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Set(
                wire,
            ) => {
                let alternative_path = child_path(path, "Set");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "element")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ModelingType,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.element.into_option(),
                            &child_path(path, "element"),
                        )?,
                        &child_path(path, "element"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Continuous(
                wire,
            ) => {
                let alternative_path = child_path(path, "Continuous");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "domain"), (2, "element")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.domain.into_option(),
                            &child_path(path, "domain"),
                        )?,
                        &child_path(path, "domain"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ModelingType,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.element.into_option(),
                            &child_path(path, "element"),
                        )?,
                        &child_path(path, "element"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Tuple(
                wire,
            ) => {
                let alternative_path = child_path(path, "Tuple");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        17,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
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
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from_in_scope::<
                                    _,
                                    crate::host_operations::engineering::inspection::types::ModelingType,
                                    crate::host_operations::engineering::NativeAdapterScope,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        17,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Table(
                wire,
            ) => {
                let alternative_path = child_path(path, "Table");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        18,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        18,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        18,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Row(
                wire,
            ) => {
                let alternative_path = child_path(path, "Row");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        19,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        19,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        19,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Definition(
                wire,
            ) => {
                let alternative_path = child_path(path, "Definition");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        20,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        20,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        20,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Interface(
                wire,
            ) => {
                let alternative_path = child_path(path, "Interface");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        21,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        21,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "declaration")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.declaration.into_option(),
                            &child_path(path, "declaration"),
                        )?,
                        &child_path(path, "declaration"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        21,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Function(
                wire,
            ) => {
                let alternative_path = child_path(path, "Function");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        22,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
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
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from_in_scope::<
                                    _,
                                    crate::host_operations::engineering::inspection::types::Tuple2OfStringAndModelingType,
                                    crate::host_operations::engineering::NativeAdapterScope,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ModelingType,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.result.into_option(),
                            &child_path(path, "result"),
                        )?,
                        &child_path(path, "result"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        22,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Optional(
                wire,
            ) => {
                let alternative_path = child_path(path, "Optional");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        23,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        23,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "element")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ModelingType,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.element.into_option(),
                            &child_path(path, "element"),
                        )?,
                        &child_path(path, "element"),
                    )?,
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        23,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::modeling_type::Kind::Indexed(
                wire,
            ) => {
                let alternative_path = child_path(path, "Indexed");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        24,
                    >>::VERSION,
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        24,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "element"), (2, "axes")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ModelingType,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.element.into_option(),
                            &child_path(path, "element"),
                        )?,
                        &child_path(path, "element"),
                    )?,
                    {
                        let current_path = &child_path(path, "axes");
                        let values = wire
                            .axes
                            .into_iter()
                            .enumerate()
                            .map(|(index, value)| -> Result<_, AdapterError> {
                                native_from_in_scope::<
                                    _,
                                    crate::host_operations::engineering::inspection::types::DeclarationId,
                                    crate::host_operations::engineering::NativeAdapterScope,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::Type as pse_contract::boundary_projection::ProjectionAlternative<
                        24,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::NativeStartPolicy,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::enums::NativeStartPolicy {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::NativeStartPolicy,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::inspection::types::NativeStartPolicy {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::NativeStartPolicy,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_model::generated::enums::NativeStartPolicy {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::NativeStartPolicy,
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
    crate::host_operations::engineering::inspection::types::OperationSource,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::OperationSource {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::OperationSource,
        AdapterError,
    > {
        layout(
            <crate::workflow::OperationSource as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::OperationSource as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "research"), (2, "context"), (3, "revision"), (4, "physical_context")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::OperationSource {
            research: field_0
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ResearchSource,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "research"))
                })
                .transpose()?
                .into(),
            context: field_1
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::CanonicalContextSelection,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "context"))
                })
                .transpose()?
                .into(),
            revision: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::SourceRevisionHash,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_2, &child_path(path, "revision"))?,
            ),
            physical_context: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_3,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::OperationSource,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::OperationSource {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::OperationSource,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::OperationSource as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::OperationSource as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "research"), (2, "context"), (3, "revision"), (4, "physical_context")],
            path,
        )?;
        let fields = (
            wire
                .research
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ResearchSource,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "research"))
                })
                .transpose()?,
            wire
                .context
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::CanonicalContextSelection,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "context"))
                })
                .transpose()?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::SourceRevisionHash,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.revision.into_option(), &child_path(path, "revision"))?,
                &child_path(path, "revision"),
            )?,
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.physical_context,
                &child_path(path, "physical_context"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::PhysicalContractFacts,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::ContractWire {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::PhysicalContractFacts,
        AdapterError,
    > {
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
        ) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::PhysicalContractFacts {
            named: field_0
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "named"))
                })
                .transpose()?
                .into(),
            kinds: {
                let current_path = &child_path(path, "kinds");
                items(field_1)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::KindFactor,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            qualified: {
                let current_path = &child_path(path, "qualified");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::PhysicalQualifiedFactor,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            axes: {
                let current_path = &child_path(path, "axes");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::EntityKindId,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            indices: {
                let current_path = &child_path(path, "indices");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::BoundIndexReference,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            factors: {
                let current_path = &child_path(path, "factors");
                items(field_5)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::UnitFactor,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            dimension: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::PhysicalDimensionVector,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_6, &child_path(path, "dimension"))?,
            ),
            scale_bits: native_into::<_, u64>(field_7, &child_path(path, "scale_bits"))?,
            pure_number: native_into::<
                _,
                bool,
            >(field_8, &child_path(path, "pure_number"))?,
            formula: field_9
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalFormulaFacts,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "formula"))
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::PhysicalContractFacts,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::ContractWire {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::PhysicalContractFacts,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "named"))
                })
                .transpose()?,
            {
                let current_path = &child_path(path, "kinds");
                let values = wire
                    .kinds
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::KindFactor,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "qualified");
                let values = wire
                    .qualified
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::PhysicalQualifiedFactor,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "axes");
                let values = wire
                    .axes
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::EntityKindId,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "indices");
                let values = wire
                    .indices
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::BoundIndexReference,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "factors");
                let values = wire
                    .factors
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::UnitFactor,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::PhysicalDimensionVector,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.dimension.into_option(), &child_path(path, "dimension"))?,
                &child_path(path, "dimension"),
            )?,
            native_from::<_, u64>(wire.scale_bits, &child_path(path, "scale_bits"))?,
            native_from::<_, bool>(wire.pure_number, &child_path(path, "pure_number"))?,
            wire
                .formula
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalFormulaFacts,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "formula"))
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
    crate::host_operations::engineering::inspection::types::PhysicalContractRecord,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::ContractRecord {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::PhysicalContractRecord,
        AdapterError,
    > {
        layout(
            <pse_quantity::resolved::receipts::ContractRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::ContractRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::PhysicalContractRecord {
            value: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::PhysicalContractFacts,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "value"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::PhysicalContractRecord,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::ContractRecord {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::PhysicalContractRecord,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::ContractRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::ContractRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::PhysicalContractFacts,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.value.into_option(), &child_path(path, "value"))?,
                &child_path(path, "value"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::PhysicalDimensionVector,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::dimension::DimensionVector {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::PhysicalDimensionVector,
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
        Ok(crate::host_operations::engineering::inspection::types::PhysicalDimensionVector {
            exponents: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::Array8PhysicalRatio,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "exponents"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::PhysicalDimensionVector,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::dimension::DimensionVector {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::PhysicalDimensionVector,
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
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::Array8PhysicalRatio,
                crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::PhysicalFormulaFacts,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::FormulaWire {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::PhysicalFormulaFacts,
        AdapterError,
    > {
        layout(
            <pse_quantity::resolved::receipts::FormulaWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::FormulaWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "authority"), (2, "operation"), (3, "operands"), (4, "powers")],
            path,
        )?;
        let (field_0, field_1, field_2, field_3) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::PhysicalFormulaFacts {
            authority: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                field_0,
            ),
            operation: native_into::<
                _,
                String,
            >(field_1, &child_path(path, "operation"))?,
            operands: {
                let current_path = &child_path(path, "operands");
                items(field_2)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::PhysicalContractFacts,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            powers: {
                let current_path = &child_path(path, "powers");
                items(field_3)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::PhysicalRatio,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::PhysicalFormulaFacts,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::FormulaWire {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::PhysicalFormulaFacts,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::FormulaWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::FormulaWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "authority"), (2, "operation"), (3, "operands"), (4, "powers")],
            path,
        )?;
        let fields = (
            pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                wire.authority,
                &child_path(path, "authority"),
            )?,
            native_from::<_, String>(wire.operation, &child_path(path, "operation"))?,
            {
                let current_path = &child_path(path, "operands");
                let values = wire
                    .operands
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::PhysicalContractFacts,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            {
                let current_path = &child_path(path, "powers");
                let values = wire
                    .powers
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::PhysicalRatio,
                            crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::PhysicalQualifiedFactor,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::QualifiedFactorWire {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::PhysicalQualifiedFactor,
        AdapterError,
    > {
        layout(
            <pse_quantity::resolved::receipts::QualifiedFactorWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::QualifiedFactorWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "key"), (2, "exponent")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::PhysicalQualifiedFactor {
            key: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::QuantityTypeKey,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "key"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::PhysicalRatio,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::PhysicalQualifiedFactor,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::QualifiedFactorWire {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::PhysicalQualifiedFactor,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::QualifiedFactorWire as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::QualifiedFactorWire as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "key"), (2, "exponent")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::QuantityTypeKey,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.key.into_option(), &child_path(path, "key"))?,
                &child_path(path, "key"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::PhysicalRatio,
                crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::PhysicalRatio,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::dimension::Ratio {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::PhysicalRatio,
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
        Ok(crate::host_operations::engineering::inspection::types::PhysicalRatio {
            num: native_into::<_, i32>(field_0, &child_path(path, "num"))?,
            den: native_into::<_, i32>(field_1, &child_path(path, "den"))?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::PhysicalRatio,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::dimension::Ratio {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::PhysicalRatio,
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
    crate::host_operations::engineering::inspection::types::PhysicalSchemeRecord,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::SchemeRecord {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::PhysicalSchemeRecord,
        AdapterError,
    > {
        layout(
            <pse_quantity::resolved::receipts::SchemeRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::SchemeRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::PhysicalSchemeRecord {
            value: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "value"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::PhysicalSchemeRecord,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::SchemeRecord {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::PhysicalSchemeRecord,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::resolved::receipts::SchemeRecord as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::resolved::receipts::SchemeRecord as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.value.into_option(), &child_path(path, "value"))?,
                &child_path(path, "value"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::SchemeWire {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
        AdapterError,
    > {
        let value = self;
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
        let value = match <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Concrete");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::PhysicalSchemeShape {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Concrete(
                            Box::new(crate::host_operations::engineering::inspection::types::physical_scheme_shape::Alternative1 {
                                quantity: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "quantity"))?,
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
        let value = match <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Resolved");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::PhysicalSchemeShape {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Resolved(
                            Box::new(crate::host_operations::engineering::inspection::types::physical_scheme_shape::Alternative2 {
                                contract: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::PhysicalContractRecord,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "contract"))?,
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
        let value = match <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Variable");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::PhysicalSchemeShape {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Variable(
                            Box::new(crate::host_operations::engineering::inspection::types::physical_scheme_shape::Alternative3 {
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
        let value = match <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Delta");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::PhysicalSchemeShape {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Delta(
                            Box::new(crate::host_operations::engineering::inspection::types::physical_scheme_shape::Alternative4 {
                                value: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
        let value = match <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Product");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::PhysicalSchemeShape {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Product(
                            Box::new(crate::host_operations::engineering::inspection::types::physical_scheme_shape::Alternative5 {
                                left: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "left"))?,
                                ),
                                right: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "right"))?,
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
        let value = match <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Quotient");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::PhysicalSchemeShape {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Quotient(
                            Box::new(crate::host_operations::engineering::inspection::types::physical_scheme_shape::Alternative6 {
                                left: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "left"))?,
                                ),
                                right: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "right"))?,
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
        let value = match <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
            7,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Power");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::PhysicalSchemeShape {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Power(
                            Box::new(crate::host_operations::engineering::inspection::types::physical_scheme_shape::Alternative7 {
                                value: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "value"))?,
                                ),
                                power: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::PhysicalRatio,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_1, &child_path(path, "power"))?,
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
    crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::resolved::receipts::SchemeWire {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Concrete(
                wire,
            ) => {
                let alternative_path = child_path(path, "Concrete");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.quantity.into_option(),
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Resolved(
                wire,
            ) => {
                let alternative_path = child_path(path, "Resolved");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalContractRecord,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.contract.into_option(),
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Variable(
                wire,
            ) => {
                let alternative_path = child_path(path, "Variable");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
                    native_from::<_, String>(wire.name, &child_path(path, "name"))?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Delta(
                wire,
            ) => {
                let alternative_path = child_path(path, "Delta");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Product(
                wire,
            ) => {
                let alternative_path = child_path(path, "Product");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.left.into_option(), &child_path(path, "left"))?,
                        &child_path(path, "left"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.right.into_option(), &child_path(path, "right"))?,
                        &child_path(path, "right"),
                    )?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Quotient(
                wire,
            ) => {
                let alternative_path = child_path(path, "Quotient");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.left.into_option(), &child_path(path, "left"))?,
                        &child_path(path, "left"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.right.into_option(), &child_path(path, "right"))?,
                        &child_path(path, "right"),
                    )?,
                );
                Ok(
                    <pse_quantity::resolved::receipts::SchemeWire as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::physical_scheme_shape::Kind::Power(
                wire,
            ) => {
                let alternative_path = child_path(path, "Power");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
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
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalSchemeShape,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::PhysicalRatio,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.power.into_option(), &child_path(path, "power"))?,
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
impl NativeInto<
    crate::host_operations::engineering::inspection::types::PreparationSettings,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::PreparationSettings {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::PreparationSettings,
        AdapterError,
    > {
        layout(
            <crate::workflow::PreparationSettings as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PreparationSettings as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "compiler"), (2, "limits")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::PreparationSettings {
            compiler: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::CompilerProfile,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "compiler"))?,
            ),
            limits: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingLimits,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "limits"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::PreparationSettings,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::PreparationSettings {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::PreparationSettings,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::PreparationSettings as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::PreparationSettings as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "compiler"), (2, "limits")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::CompilerProfile,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.compiler.into_option(), &child_path(path, "compiler"))?,
                &child_path(path, "compiler"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingLimits,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.limits.into_option(), &child_path(path, "limits"))?,
                &child_path(path, "limits"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::QuantityKindId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::QuantityKindId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::QuantityKindId,
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
        Ok(crate::host_operations::engineering::inspection::types::QuantityKindId {
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
    crate::host_operations::engineering::inspection::types::QuantityKindId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::QuantityKindId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::QuantityKindId,
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
    crate::host_operations::engineering::inspection::types::QuantityScheme,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::scheme::Scheme {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::QuantityScheme,
        AdapterError,
    > {
        layout(
            <pse_quantity::scheme::Scheme as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::scheme::Scheme as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "document")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::QuantityScheme {
            document: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::PhysicalSchemeRecord,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "document"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::QuantityScheme,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::scheme::Scheme {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::QuantityScheme,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_quantity::scheme::Scheme as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_quantity::scheme::Scheme as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "document")],
            path,
        )?;
        let fields = (
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::PhysicalSchemeRecord,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.document.into_option(), &child_path(path, "document"))?,
                &child_path(path, "document"),
            )?,
        );
        pse_contract::boundary_projection::ProjectionRecord::from_projection_fields(
                fields,
            )
            .map_err(|error| error.at(path))
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::QuantityTypeId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::QuantityTypeId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::QuantityTypeId,
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
        Ok(crate::host_operations::engineering::inspection::types::QuantityTypeId {
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
    crate::host_operations::engineering::inspection::types::QuantityTypeId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::QuantityTypeId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::QuantityTypeId,
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
    crate::host_operations::engineering::inspection::types::QuantityTypeKey,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::QuantityTypeKey,
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
        Ok(crate::host_operations::engineering::inspection::types::QuantityTypeKey {
            kind: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::QuantityKindId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "kind"))?,
            ),
            basis: field_1
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::BasisId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "basis"))
                })
                .transpose()?
                .into(),
            reference_state: field_2
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ReferenceStateId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "reference_state"))
                })
                .transpose()?
                .into(),
            scale_kind: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ScaleKind,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_3, &child_path(path, "scale_kind"))?,
            ),
            shape: {
                let current_path = &child_path(path, "shape");
                items(field_4)
                    .enumerate()
                    .map(|(index, value)| -> Result<_, AdapterError> {
                        native_into_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::EntityKindId,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?
            },
            subject_kind: field_5
                .map(|value| -> Result<_, AdapterError> {
                    native_into_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::EntityKindId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "subject_kind"))
                })
                .transpose()?
                .into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::QuantityTypeKey,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::quantity_type::QuantityTypeKey {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::QuantityTypeKey,
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
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::QuantityKindId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.kind.into_option(), &child_path(path, "kind"))?,
                &child_path(path, "kind"),
            )?,
            wire
                .basis
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::BasisId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "basis"))
                })
                .transpose()?,
            wire
                .reference_state
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ReferenceStateId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "reference_state"))
                })
                .transpose()?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ScaleKind,
                crate::host_operations::engineering::NativeAdapterScope,
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
                        native_from_in_scope::<
                            _,
                            crate::host_operations::engineering::inspection::types::EntityKindId,
                            crate::host_operations::engineering::NativeAdapterScope,
                        >(value, &format!("{path}[{index}]", path = current_path))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                from_items(values, current_path)?
            },
            wire
                .subject_kind
                .into_option()
                .map(|value| -> Result<_, AdapterError> {
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::EntityKindId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(value, &child_path(path, "subject_kind"))
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
    crate::host_operations::engineering::inspection::types::ReferenceStateId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::ReferenceStateId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ReferenceStateId,
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
        Ok(crate::host_operations::engineering::inspection::types::ReferenceStateId {
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
    crate::host_operations::engineering::inspection::types::ReferenceStateId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::ReferenceStateId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ReferenceStateId,
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
    crate::host_operations::engineering::inspection::types::ResearchSource,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::ResearchSource {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ResearchSource,
        AdapterError,
    > {
        layout(
            <crate::workflow::ResearchSource as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::ResearchSource as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "issuing_store"),
                (2, "selection"),
                (3, "dataset"),
                (4, "revision"),
                (5, "physical_context"),
                (6, "publication"),
            ],
            path,
        )?;
        let (field_0, field_1, field_2, field_3, field_4, field_5) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::ResearchSource {
            issuing_store: native_into::<
                _,
                String,
            >(field_0, &child_path(path, "issuing_store"))?,
            selection: native_into::<
                _,
                String,
            >(field_1, &child_path(path, "selection"))?,
            dataset: native_into::<_, String>(field_2, &child_path(path, "dataset"))?,
            revision: native_into::<_, String>(field_3, &child_path(path, "revision"))?,
            physical_context: native_into::<
                _,
                String,
            >(field_4, &child_path(path, "physical_context"))?,
            publication: field_5
                .map(|value| -> Result<_, AdapterError> {
                    native_into::<_, String>(value, &child_path(path, "publication"))
                })
                .transpose()?,
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ResearchSource,
    crate::host_operations::engineering::NativeAdapterScope,
> for crate::workflow::ResearchSource {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ResearchSource,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <crate::workflow::ResearchSource as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <crate::workflow::ResearchSource as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[
                (1, "issuing_store"),
                (2, "selection"),
                (3, "dataset"),
                (4, "revision"),
                (5, "physical_context"),
                (6, "publication"),
            ],
            path,
        )?;
        let fields = (
            native_from::<
                _,
                String,
            >(wire.issuing_store, &child_path(path, "issuing_store"))?,
            native_from::<_, String>(wire.selection, &child_path(path, "selection"))?,
            native_from::<_, String>(wire.dataset, &child_path(path, "dataset"))?,
            native_from::<_, String>(wire.revision, &child_path(path, "revision"))?,
            native_from::<
                _,
                String,
            >(wire.physical_context, &child_path(path, "physical_context"))?,
            wire
                .publication
                .map(|value| -> Result<_, AdapterError> {
                    native_from::<_, String>(value, &child_path(path, "publication"))
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
    crate::host_operations::engineering::inspection::types::ScaleKind,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::enums::ScaleKind {
    fn into_wire(
        self,
        _: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::ScaleKind,
        AdapterError,
    > {
        Ok(crate::host_operations::engineering::inspection::types::ScaleKind {
            value: self.as_str().into(),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::ScaleKind,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::enums::ScaleKind {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::ScaleKind,
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
    crate::host_operations::engineering::inspection::types::SourceRevisionHash,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_ids::roles::SourceRevisionHash {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::SourceRevisionHash,
        AdapterError,
    > {
        layout(
            <pse_ids::roles::SourceRevisionHash as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_ids::roles::SourceRevisionHash as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "id")],
            path,
        )?;
        let (field_0,) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::SourceRevisionHash {
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
    crate::host_operations::engineering::inspection::types::SourceRevisionHash,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_ids::roles::SourceRevisionHash {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::SourceRevisionHash,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <pse_ids::roles::SourceRevisionHash as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <pse_ids::roles::SourceRevisionHash as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
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
    crate::host_operations::engineering::inspection::types::StructuralValue,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::specialize::Value {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::StructuralValue,
        AdapterError,
    > {
        let value = self;
        layout(
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                1,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            1,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Boolean(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative1 {
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                2,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            2,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Integer(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative2 {
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                3,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "bits"), (2, "quantity")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            3,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Number");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Number(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative3 {
                                bits: native_into::<
                                    _,
                                    u64,
                                >(field_0, &child_path(path, "bits"))?,
                                quantity: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                4,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "bits"), (3, "quantity")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            4,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1, field_2)) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Coordinate(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative4 {
                                id: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                                    field_0,
                                ),
                                bits: native_into::<
                                    _,
                                    u64,
                                >(field_1, &child_path(path, "bits"))?,
                                quantity: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                5,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            5,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Text(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative5 {
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                6,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "kind")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            6,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Entity");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Entity(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative6 {
                                id: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "id"))?,
                                ),
                                kind: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                7,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "enumeration"), (2, "member")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            7,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Enum");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Enum(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative7 {
                                enumeration: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "enumeration"))?,
                                ),
                                member: pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes(
                                    field_1,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                8,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "scheme"), (2, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            8,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Identifier");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Identifier(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative8 {
                                scheme: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                9,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                9,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "id"), (2, "bindings")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            9,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1)) => {
                let alternative_path = child_path(path, "Definition");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Definition(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative9 {
                                id: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
                                                native_into_in_scope::<
                                                    _,
                                                    crate::host_operations::engineering::inspection::types::StructuralValue,
                                                    crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                10,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                10,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            10,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Function");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Function(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative10 {
                                value: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                11,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                11,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "table"), (2, "names"), (3, "fields")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            11,
        >>::into_projection_alternative(value) {
            Ok((field_0, field_1, field_2)) => {
                let alternative_path = child_path(path, "Row");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Row(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative11 {
                                table: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::DeclarationId,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(field_0, &child_path(path, "table"))?,
                                ),
                                names: {
                                    let current_path = &child_path(path, "names");
                                    items(field_1)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into::<
                                                _,
                                                String,
                                            >(value, &format!("{path}[{index}]", path = current_path))
                                        })
                                        .collect::<Result<Vec<_>, _>>()?
                                },
                                fields: {
                                    let current_path = &child_path(path, "fields");
                                    items(field_2)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into_in_scope::<
                                                _,
                                                crate::host_operations::engineering::inspection::types::StructuralValue,
                                                crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                12,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                12,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            12,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Set");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Set(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative12 {
                                value: {
                                    let current_path = &child_path(path, "value");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into_in_scope::<
                                                _,
                                                crate::host_operations::engineering::inspection::types::StructuralValue,
                                                crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                13,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                13,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            13,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "Tuple");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Tuple(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative13 {
                                value: {
                                    let current_path = &child_path(path, "value");
                                    items(field_0)
                                        .enumerate()
                                        .map(|(index, value)| -> Result<_, AdapterError> {
                                            native_into_in_scope::<
                                                _,
                                                crate::host_operations::engineering::inspection::types::StructuralValue,
                                                crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                14,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                14,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            14,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "QuantityType");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::QuantityType(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative14 {
                                value: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                15,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                15,
            >>::FIELD_LAYOUT,
            1,
            &[(1, "value")],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            15,
        >>::into_projection_alternative(value) {
            Ok((field_0,)) => {
                let alternative_path = child_path(path, "ReferenceState");
                let path = alternative_path.as_str();
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::ReferenceState(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative15 {
                                value: buffa::MessageField::some(
                                    native_into_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::ReferenceStateId,
                                        crate::host_operations::engineering::NativeAdapterScope,
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
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                16,
            >>::VERSION,
            <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                16,
            >>::FIELD_LAYOUT,
            1,
            &[],
            path,
        )?;
        let value = match <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
            16,
        >>::into_projection_alternative(value) {
            Ok(()) => {
                return Ok(crate::host_operations::engineering::inspection::types::StructuralValue {
                    kind: Some(
                        crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Missing(
                            Box::new(crate::host_operations::engineering::inspection::types::structural_value::Alternative16 {
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
    crate::host_operations::engineering::inspection::types::StructuralValue,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_modeling::specialize::Value {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::StructuralValue,
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
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Boolean(
                wire,
            ) => {
                let alternative_path = child_path(path, "Boolean");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        1,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Integer(
                wire,
            ) => {
                let alternative_path = child_path(path, "Integer");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        2,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Number(
                wire,
            ) => {
                let alternative_path = child_path(path, "Number");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "bits"), (2, "quantity")],
                    path,
                )?;
                let fields = (
                    native_from::<_, u64>(wire.bits, &child_path(path, "bits"))?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.quantity.into_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        3,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Coordinate(
                wire,
            ) => {
                let alternative_path = child_path(path, "Coordinate");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "bits"), (3, "quantity")],
                    path,
                )?;
                let fields = (
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        wire.id,
                        &child_path(path, "id"),
                    )?,
                    native_from::<_, u64>(wire.bits, &child_path(path, "bits"))?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.quantity.into_option(),
                            &child_path(path, "quantity"),
                        )?,
                        &child_path(path, "quantity"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        4,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Text(
                wire,
            ) => {
                let alternative_path = child_path(path, "Text");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
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
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        5,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Entity(
                wire,
            ) => {
                let alternative_path = child_path(path, "Entity");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "kind")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.id.into_option(), &child_path(path, "id"))?,
                        &child_path(path, "id"),
                    )?,
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.kind.into_option(), &child_path(path, "kind"))?,
                        &child_path(path, "kind"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        6,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Enum(
                wire,
            ) => {
                let alternative_path = child_path(path, "Enum");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "enumeration"), (2, "member")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(
                            wire.enumeration.into_option(),
                            &child_path(path, "enumeration"),
                        )?,
                        &child_path(path, "enumeration"),
                    )?,
                    pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes(
                        wire.member,
                        &child_path(path, "member"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        7,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Identifier(
                wire,
            ) => {
                let alternative_path = child_path(path, "Identifier");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "scheme"), (2, "value")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
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
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        8,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Definition(
                wire,
            ) => {
                let alternative_path = child_path(path, "Definition");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "id"), (2, "bindings")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
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
                                    native_from_in_scope::<
                                        _,
                                        crate::host_operations::engineering::inspection::types::StructuralValue,
                                        crate::host_operations::engineering::NativeAdapterScope,
                                    >(value, &child_path(path, "bindings"))?,
                                ))
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                        &child_path(path, "bindings"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        9,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Function(
                wire,
            ) => {
                let alternative_path = child_path(path, "Function");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        10,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Row(
                wire,
            ) => {
                let alternative_path = child_path(path, "Row");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "table"), (2, "names"), (3, "fields")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::DeclarationId,
                        crate::host_operations::engineering::NativeAdapterScope,
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
                                native_from::<
                                    _,
                                    String,
                                >(value, &format!("{path}[{index}]", path = current_path))
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
                                native_from_in_scope::<
                                    _,
                                    crate::host_operations::engineering::inspection::types::StructuralValue,
                                    crate::host_operations::engineering::NativeAdapterScope,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        11,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Set(
                wire,
            ) => {
                let alternative_path = child_path(path, "Set");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
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
                                native_from_in_scope::<
                                    _,
                                    crate::host_operations::engineering::inspection::types::StructuralValue,
                                    crate::host_operations::engineering::NativeAdapterScope,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        12,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Tuple(
                wire,
            ) => {
                let alternative_path = child_path(path, "Tuple");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
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
                                native_from_in_scope::<
                                    _,
                                    crate::host_operations::engineering::inspection::types::StructuralValue,
                                    crate::host_operations::engineering::NativeAdapterScope,
                                >(value, &format!("{path}[{index}]", path = current_path))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        from_items(values, current_path)?
                    },
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        13,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::QuantityType(
                wire,
            ) => {
                let alternative_path = child_path(path, "QuantityType");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::QuantityTypeId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        14,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::ReferenceState(
                wire,
            ) => {
                let alternative_path = child_path(path, "ReferenceState");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::FIELD_LAYOUT,
                    1,
                    &[(1, "value")],
                    path,
                )?;
                let fields = (
                    native_from_in_scope::<
                        _,
                        crate::host_operations::engineering::inspection::types::ReferenceStateId,
                        crate::host_operations::engineering::NativeAdapterScope,
                    >(
                        required(wire.value.into_option(), &child_path(path, "value"))?,
                        &child_path(path, "value"),
                    )?,
                );
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        15,
                    >>::from_projection_alternative(fields),
                )
            }
            crate::host_operations::engineering::inspection::types::__buffa::oneof::structural_value::Kind::Missing(
                wire,
            ) => {
                let alternative_path = child_path(path, "Missing");
                let path = alternative_path.as_str();
                let wire = *wire;
                unknown_fields(&wire.__buffa_unknown_fields, path)?;
                layout(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::VERSION,
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::FIELD_LAYOUT,
                    1,
                    &[],
                    path,
                )?;
                let fields = ();
                Ok(
                    <pse_modeling::specialize::Value as pse_contract::boundary_projection::ProjectionAlternative<
                        16,
                    >>::from_projection_alternative(fields),
                )
            }
        }
    }
}
impl NativeInto<
    crate::host_operations::engineering::inspection::types::Tuple2OfStringAndModelingType,
    crate::host_operations::engineering::NativeAdapterScope,
> for (String, pse_modeling::Type) {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::Tuple2OfStringAndModelingType,
        AdapterError,
    > {
        layout(
            <(
                String,
                pse_modeling::Type,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                String,
                pse_modeling::Type,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let (field_0, field_1) = pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(
            self,
        );
        Ok(crate::host_operations::engineering::inspection::types::Tuple2OfStringAndModelingType {
            first: native_into::<_, String>(field_0, &child_path(path, "first"))?,
            second: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::ModelingType,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "second"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::Tuple2OfStringAndModelingType,
    crate::host_operations::engineering::NativeAdapterScope,
> for (String, pse_modeling::Type) {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::Tuple2OfStringAndModelingType,
        path: &str,
    ) -> Result<Self, AdapterError> {
        unknown_fields(&wire.__buffa_unknown_fields, path)?;
        layout(
            <(
                String,
                pse_modeling::Type,
            ) as pse_contract::boundary_projection::ProjectionRecord>::VERSION,
            <(
                String,
                pse_modeling::Type,
            ) as pse_contract::boundary_projection::ProjectionRecord>::FIELD_LAYOUT,
            1,
            &[(1, "first"), (2, "second")],
            path,
        )?;
        let fields = (
            native_from::<_, String>(wire.first, &child_path(path, "first"))?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::ModelingType,
                crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::UnitFactor,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::unit::UnitFactor {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::UnitFactor,
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
        Ok(crate::host_operations::engineering::inspection::types::UnitFactor {
            unit: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::UnitId,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_0, &child_path(path, "unit"))?,
            ),
            exponent: buffa::MessageField::some(
                native_into_in_scope::<
                    _,
                    crate::host_operations::engineering::inspection::types::PhysicalRatio,
                    crate::host_operations::engineering::NativeAdapterScope,
                >(field_1, &child_path(path, "exponent"))?,
            ),
            ..Default::default()
        })
    }
}
impl NativeFrom<
    crate::host_operations::engineering::inspection::types::UnitFactor,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::unit::UnitFactor {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::UnitFactor,
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
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::UnitId,
                crate::host_operations::engineering::NativeAdapterScope,
            >(
                required(wire.unit.into_option(), &child_path(path, "unit"))?,
                &child_path(path, "unit"),
            )?,
            native_from_in_scope::<
                _,
                crate::host_operations::engineering::inspection::types::PhysicalRatio,
                crate::host_operations::engineering::NativeAdapterScope,
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
    crate::host_operations::engineering::inspection::types::UnitId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::UnitId {
    fn into_wire(
        self,
        path: &str,
    ) -> Result<
        crate::host_operations::engineering::inspection::types::UnitId,
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
        Ok(crate::host_operations::engineering::inspection::types::UnitId {
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
    crate::host_operations::engineering::inspection::types::UnitId,
    crate::host_operations::engineering::NativeAdapterScope,
> for pse_quantity::ids::UnitId {
    fn from_wire(
        wire: crate::host_operations::engineering::inspection::types::UnitId,
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
/// Encode the actual native `ModelingInspection` and validate outgoing local rules.
pub fn encode_modeling_inspection(
    native: crate::workflow::ModelingInspection,
) -> Result<
    crate::host_operations::engineering::inspection::types::ModelingInspection,
    BoundaryProjectionError,
> {
    encode_root_in_scope::<
        _,
        _,
        crate::host_operations::engineering::NativeAdapterScope,
    >(native, "ModelingInspection")
}
/// Adopt known `ModelingInspection` meaning under this package's declared unknown-field policy.
pub fn decode_modeling_inspection(
    wire: crate::host_operations::engineering::inspection::types::ModelingInspection,
) -> Result<crate::workflow::ModelingInspection, BoundaryProjectionError> {
    decode_root_in_scope::<
        _,
        _,
        crate::host_operations::engineering::NativeAdapterScope,
    >(wire, "ModelingInspection")
}
