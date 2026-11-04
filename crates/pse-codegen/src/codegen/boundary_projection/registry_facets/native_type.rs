// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native source correspondence through the actual Rust renderer, not wire kinds.
use super::super::{ProjectionCodegenError, error};
use crate::codegen::native::Mode;
use crate::codegen::rust::types;
use arrow_schema::DataType;
use pse_contract::boundary_projection::{self as native, Shape, TypeRef};
use pse_contract::metadata::{EnumValue, MessageMetadata};
use pse_contract_proto::{FieldContext, MetadataError};
use pse_schema::model::{BoundaryShape, FieldContract, TaggedAlternative};
use types::qualify_projection as qualify;

fn unavailable(reason: impl Into<String>) -> ProjectionCodegenError {
    MetadataError::Unavailable {
        reason: reason.into(),
    }
    .into()
}

pub(super) fn check(
    field: &arrow_schema::Field,
    context: FieldContext<'_>,
    pool: &buffa_descriptor::DescriptorPool,
) -> Result<(), ProjectionCodegenError> {
    let declared = context.native_type.ok_or_else(|| {
        unavailable("original native field type is unavailable for IPC correspondence")
    })?;
    let rules = context.local_rules.ok_or_else(|| {
        unavailable("original native local rules are unavailable for IPC correspondence")
    })?;
    let field = FieldContract::from_field(field.clone());
    field.validate_projection_field(context.subject)?;
    if rules
        != types::field_rules(&field)?
            .into_iter()
            .map(|(rule, _)| rule)
            .collect::<Vec<_>>()
    {
        return Err(error("native IPC field contradicts its core local rules"));
    }
    let resolved = resolve(&field, declared, context, pool, Mode::Domain)?;
    types::check_native_contract(
        &resolved,
        stem(declared),
        Mode::Domain,
        declared,
        context.package,
    )?;
    Ok(())
}
fn stem(ty: &TypeRef) -> &str {
    match ty {
        TypeRef::Named(name) => name.rsplit('.').next().unwrap_or(name),
        TypeRef::Optional(value) => stem(value),
        _ => "AnonymousNativeField",
    }
}
fn value_type<'a>(
    field: &FieldContract,
    declared: &'a TypeRef,
) -> Result<&'a TypeRef, ProjectionCodegenError> {
    if field.nullable() {
        if let TypeRef::Optional(value) = declared {
            return Ok(value);
        }
        return Err(error(
            "nullable native field lacks its core Optional declaration",
        ));
    }
    Ok(declared)
}
fn owner<'a>(
    name: &str,
    context: FieldContext<'a>,
) -> Result<MessageMetadata<'a>, ProjectionCodegenError> {
    context
        .native_owners
        .iter()
        .find(|owner| owner.name == name)
        .and_then(|owner| owner.metadata)
        .ok_or_else(|| unavailable(format!("original native owner {name} is unavailable")))
}
fn named(ty: &TypeRef) -> Result<&str, ProjectionCodegenError> {
    if let TypeRef::Named(name) = ty {
        Ok(name)
    } else {
        Err(error(
            "native named source type has a contradictory core declaration",
        ))
    }
}

// Reify only the documented anonymous-name default, from the declared native
// owner. Explicit source names are never replaced. Each resolved owner is checked
// structurally below; this is not source authentication or native adoption.
fn resolve(
    field: &FieldContract,
    declared: &TypeRef,
    context: FieldContext<'_>,
    pool: &buffa_descriptor::DescriptorPool,
    mode: Mode,
) -> Result<FieldContract, ProjectionCodegenError> {
    let value = value_type(field, declared)?;
    if field.enum_name().is_some() {
        let metadata = owner(named(value)?, context)?;
        if !matches!(metadata.native_shape, Some(Shape::StringEnum(_))) {
            return Err(error(
                "native enum reference does not name its declared string vocabulary",
            ));
        }
        types::check_native_contract(field, stem(value), mode, declared, context.package)?;
        return Ok(field.clone());
    }
    if matches!(mode, Mode::Domain)
        && let Some(identity) = field.identity()
    {
        let expected = qualify(
            crate::codegen::rust::identities::projection_type(identity),
            context.package,
        );
        if expected != *value {
            return Err(error(
                "native identity association contradicts its nominal core owner",
            ));
        }
        let metadata = owner(named(value)?, context)?;
        let shape = metadata
            .native_shape
            .ok_or_else(|| unavailable("original nominal identity shape is unavailable"))?;
        let storage = types::projection_type_in(
            &field.clone().with_nullable(false),
            "IdentityStorage",
            Mode::Storage,
        )?;
        if *shape != native::identity_shape(qualify(storage, context.package)) {
            return Err(error(
                "native identity storage contradicts its nominal core record",
            ));
        }
        return Ok(field.clone());
    }
    if matches!(mode, Mode::Domain)
        && let Some((storage_stem, storage)) = types::projection_storage(field)?
    {
        let resolved = resolve(&storage, value, context, pool, Mode::Storage)?;
        types::check_native_contract(
            &resolved,
            &format!("Extension{storage_stem}"),
            Mode::Storage,
            value,
            context.package,
        )?;
        return Ok(field.clone());
    }
    let child_mode = crate::codegen::native::child_mode(field, mode);
    let resolved = match field.data_type() {
        DataType::List(child) => {
            let element = native::sequence_element(value)
                .ok_or_else(|| error("native list contradicts its core Vec representation"))?;
            let child = resolve(
                &FieldContract::from_field(child.as_ref().clone()),
                &element,
                context,
                pool,
                child_mode,
            )?;
            Ok(FieldContract::from_field(
                field
                    .field()
                    .clone()
                    .with_data_type(DataType::List(std::sync::Arc::new(child.into_field()))),
            ))
        }
        DataType::FixedSizeList(child, width) => {
            let metadata = owner(named(value)?, context)?;
            let shape = metadata
                .native_shape
                .ok_or_else(|| unavailable("original fixed-array shape is unavailable"))?;
            let Shape::Record(fields) = shape else {
                return Err(error("fixed array does not name its native record owner"));
            };
            let [values] = fields.as_slice() else {
                return Err(error("fixed-array native value layout changed"));
            };
            let element = native::sequence_element(&values.ty)
                .ok_or_else(|| error("fixed-array values are not its native Vec declaration"))?;
            let width =
                usize::try_from(width).map_err(|_| error("negative native array extent"))?;
            if *shape != native::fixed_array_shape(element.clone(), width)? {
                return Err(error(
                    "native array extent/element contradicts its core owner shape",
                ));
            }
            let child = resolve(
                &FieldContract::from_field(child.as_ref().clone()),
                &element,
                context,
                pool,
                child_mode,
            )?;
            Ok(FieldContract::from_field(
                field
                    .field()
                    .clone()
                    .with_data_type(DataType::FixedSizeList(
                        std::sync::Arc::new(child.into_field()),
                        i32::try_from(width)
                            .map_err(|_| error("native array extent exceeds Arrow width"))?,
                    )),
            ))
        }
        DataType::Struct(_) => resolve_record(field, value, context, pool, mode),
        _ => Ok(field.clone()),
    }?;
    types::check_native_contract(&resolved, stem(declared), mode, declared, context.package)?;
    Ok(resolved)
}

fn resolve_record(
    field: &FieldContract,
    declared: &TypeRef,
    context: FieldContext<'_>,
    pool: &buffa_descriptor::DescriptorPool,
    mode: Mode,
) -> Result<FieldContract, ProjectionCodegenError> {
    let name = named(declared)?;
    let metadata = owner(name, context)?;
    let declared_shape = metadata
        .native_shape
        .ok_or_else(|| unavailable("original native record shape is unavailable"))?;
    let mut presentation = BoundaryShape::from_field(field.field())?
        .ok_or_else(|| unavailable("native structure has no source projection presentation"))?;
    check_presentation(&presentation, name, metadata, context, pool)?;
    let children = field.children();
    let resolved = if let Some(alternatives) = TaggedAlternative::from_field(field.field())? {
        let Shape::Alternatives(arms) = declared_shape else {
            return Err(error("native alternatives contradict the core owner shape"));
        };
        if arms.len() != alternatives.arms.len() || presentation.alternatives.len() != arms.len() {
            return Err(error(
                "native alternative population contradicts its source presentation",
            ));
        }
        let mut resolved = children.clone();
        for arm in arms {
            if !presentation
                .alternatives
                .contains(&(arm.name.clone(), arm.tag))
            {
                return Err(error(
                    "native alternative tag/name contradicts its core owner",
                ));
            }
            let payload = alternatives
                .arms
                .get(&arm.name)
                .ok_or_else(|| error("native alternative member is absent"))?;
            if let Some(payload) = payload {
                let [value] = arm.fields.as_slice() else {
                    return Err(error("native alternative payload layout changed"));
                };
                let position = children
                    .iter()
                    .position(|field| field.name() == payload)
                    .ok_or_else(|| error("native alternative payload field is absent"))?;
                let column = children[position].clone().with_nullable(false);
                let rules = types::alternative_field_rules(&column)?
                    .into_iter()
                    .map(|(rule, _)| rule)
                    .collect::<Vec<_>>();
                if value.name != "value"
                    || value.tag != 1
                    || value.direction != presentation.direction
                    || value.rules != rules
                {
                    return Err(error(
                        "native alternative payload contradicts its core declaration",
                    ));
                }
                let payload = resolve(
                    &column,
                    &value.ty,
                    context,
                    pool,
                    crate::codegen::native::child_mode(field, mode),
                )?
                .with_nullable(true);
                resolved[position] = payload;
            } else if !arm.fields.is_empty() {
                return Err(error(
                    "native unit alternative has unexpected core payload fields",
                ));
            }
        }
        resolved
    } else {
        let Shape::Record(fields) = declared_shape else {
            return Err(error("native record contradicts its core owner shape"));
        };
        if fields.len() != children.len() {
            return Err(error(
                "native record field population contradicts its core shape",
            ));
        }
        children
            .iter()
            .map(|column| {
                let declared = fields
                    .iter()
                    .find(|field| field.name == column.name())
                    .ok_or_else(|| error("native record field is absent from its core owner"))?;
                let rules = types::field_rules(column)?
                    .into_iter()
                    .map(|(rule, _)| rule)
                    .collect::<Vec<_>>();
                if column.boundary_tag() != Some(declared.tag)
                    || declared.direction != presentation.direction
                    || declared.rules != rules
                {
                    return Err(error(
                        "native record field tag/rules/direction contradicts its core owner",
                    ));
                }
                resolve(
                    column,
                    &declared.ty,
                    context,
                    pool,
                    crate::codegen::native::child_mode(field, mode),
                )
            })
            .collect::<Result<Vec<_>, ProjectionCodegenError>>()?
    };
    if presentation.name.is_none()
        && (matches!(mode, Mode::Storage) || field.structure_name().is_none())
    {
        presentation.name = Some(stem(declared).into());
    }
    Ok(FieldContract::from_field(
        field.field().clone().with_data_type(DataType::Struct(
            resolved
                .into_iter()
                .map(FieldContract::into_field)
                .collect(),
        )),
    )
    .with_boundary_shape(presentation))
}

fn check_presentation(
    shape: &BoundaryShape,
    name: &str,
    metadata: MessageMetadata<'_>,
    context: FieldContext<'_>,
    pool: &buffa_descriptor::DescriptorPool,
) -> Result<(), ProjectionCodegenError> {
    if metadata.interpretation_version != Some(shape.version)
        || !matches!(metadata.direction, Some(EnumValue::Known(direction)) if direction == shape.direction)
        || metadata
            .owner
            .is_none_or(|owner| owner.declaration_file != shape.source)
    {
        return Err(error(
            "native presentation contradicts its original core owner interpretation/source/direction",
        ));
    }
    let owner = context
        .native_owners
        .iter()
        .find(|owner| owner.name == name)
        .ok_or_else(|| unavailable("original native presentation owner is unavailable"))?;
    let file = pool
        .file_by_name(owner.file)
        .ok_or_else(|| unavailable("original native presentation file is unavailable"))?;
    let descriptor = file
        .message_type
        .iter()
        .find(|message| {
            format!(
                "{}.{}",
                file.package.as_deref().unwrap_or_default(),
                message.name.as_deref().unwrap_or_default()
            ) == name
        })
        .ok_or_else(|| unavailable("original native presentation descriptor is unavailable"))?;
    let expected = shape
        .reserved_tags
        .iter()
        .map(|tag| (Some(*tag as i32), Some((*tag + 1) as i32)))
        .collect::<std::collections::BTreeSet<_>>();
    let actual = descriptor
        .reserved_range
        .iter()
        .map(|range| (range.start, range.end))
        .collect::<std::collections::BTreeSet<_>>();
    if expected != actual
        || shape
            .reserved_names
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            != descriptor
                .reserved_name
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
    {
        return Err(error(
            "native presentation reservations contradict its original owner",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use native::{Direction, LocalRule, ProjectionError, ProjectionGraph, ProjectionType};
    use pse_contract::metadata::{Message, Owner};
    const PACKAGE: &str = "pse.correspondence.v1";

    fn qualified_shape(shape: &Shape) -> Shape {
        let field = |value: &native::Field| native::Field {
            ty: qualify(value.ty.clone(), PACKAGE),
            ..value.clone()
        };
        match shape {
            Shape::Record(fields) => Shape::Record(fields.iter().map(field).collect()),
            Shape::Alternatives(arms) => Shape::Alternatives(
                arms.iter()
                    .map(|arm| native::Alternative {
                        fields: arm.fields.iter().map(field).collect(),
                        ..arm.clone()
                    })
                    .collect(),
            ),
            Shape::StringEnum(values) => Shape::StringEnum(values.clone()),
        }
    }
    fn checks<T: ProjectionType>(field: &FieldContract) -> Result<(), ProjectionCodegenError> {
        let mut graph = ProjectionGraph::of::<NativeRecord>().unwrap();
        let ty = qualify(T::projection(&mut graph).unwrap(), PACKAGE);
        let descriptors = crate::codegen::boundary_projection::descriptors_with_imports(
            &graph,
            PACKAGE,
            "correspondence.proto",
            &crate::codegen::boundary_projection::tests::imports(),
        )
        .unwrap();
        let encoder = pse_contract_proto::OptionEncoder::new(&descriptors.file).unwrap();
        let names = graph
            .definitions()
            .values()
            .map(|definition| format!("{PACKAGE}.{}", definition.name))
            .collect::<Vec<_>>();
        let shapes = graph
            .definitions()
            .values()
            .map(|definition| qualified_shape(&definition.shape))
            .collect::<Vec<_>>();
        let owners = graph
            .definitions()
            .values()
            .zip(&names)
            .zip(&shapes)
            .map(|((definition, name), shape)| Message {
                name,
                file: "correspondence.proto",
                metadata: Some(MessageMetadata {
                    metadata_version: Some(2),
                    owner: Some(Owner {
                        rust_type: definition.rust_type,
                        declaration_file: definition.source,
                    }),
                    interpretation_version: Some(definition.version),
                    direction: Some(EnumValue::Known(definition.direction)),
                    construction: Some(EnumValue::Known(definition.construction)),
                    effects: None,
                    native_shape: Some(shape),
                }),
            })
            .collect::<Vec<_>>();
        let rules = types::field_rules(field)?
            .into_iter()
            .map(|(rule, _)| rule)
            .collect::<Vec<_>>();
        check(
            field.field(),
            FieldContext {
                package: PACKAGE,
                subject: "pse.correspondence.v1.Owner.value",
                native_type: Some(&ty),
                local_rules: Some(&rules),
                native_owners: &owners,
            },
            &encoder.pool(),
        )
    }

    #[test]
    fn original_native_scalar_collection_array_and_identity_correspondence_is_checked() {
        checks::<i64>(&FieldContract::native(DataType::Timestamp(
            arrow_schema::TimeUnit::Microsecond,
            None,
        )))
        .unwrap();
        assert!(checks::<u64>(&FieldContract::native(DataType::Int64)).is_err());
        checks::<pse_ids::SemanticId>(&FieldContract::id()).unwrap();
        assert!(checks::<pse_ids::SemanticId>(&FieldContract::native(DataType::Binary)).is_err());
        checks::<pse_quantity::UnitId>(&FieldContract::id().with_identity("unit")).unwrap();
        assert!(
            checks::<pse_quantity::UnitId>(&FieldContract::hash().with_identity("unit")).is_err()
        );
        assert!(
            checks::<pse_quantity::UnitId>(&FieldContract::id().with_identity("basis")).is_err()
        );
        checks::<pse_model::generated::enums::PresolvePass>(&FieldContract::enumeration(
            "PresolvePass",
        ))
        .unwrap();
        checks::<pse_model::generated::extension_values::QuantityValue>(&FieldContract::extended(
            crate::model::ExtensionUse::QuantityValue,
        ))
        .unwrap();
        let array = FieldContract::fixed_list(
            FieldContract::list(FieldContract::native(DataType::UInt8).with_name("item"))
                .with_name("item")
                .optional(),
            2,
        );
        checks::<[Option<Vec<u8>>; 2]>(&array).unwrap();
        assert!(checks::<[Option<Vec<u8>>; 3]>(&array).is_err());
        assert!(checks::<[Vec<u8>; 2]>(&array).is_err());
        checks::<[Option<pse_model::Bytes>; 2]>(&FieldContract::fixed_list(
            FieldContract::native(DataType::Binary)
                .with_name("item")
                .optional(),
            2,
        ))
        .unwrap();
        assert!(
            checks::<Vec<u64>>(&FieldContract::list(
                FieldContract::native(DataType::Int64).with_name("item")
            ))
            .is_err()
        );
    }

    #[test]
    fn every_quantity_storage_child_has_its_own_explicit_native_contract() {
        let original =
            ProjectionGraph::of::<pse_model::generated::extension_values::QuantityValue>().unwrap();
        let definition = &original.definitions()["ExtensionQuantityValue"];
        let storage = crate::model::ExtensionUse::QuantityValue
            .boundary_storage()
            .unwrap();
        struct PresentedStorage;
        let graph = |changed: Option<FieldContract>| {
            let Shape::Record(mut fields) = definition.shape.clone() else {
                panic!("actual native storage record");
            };
            for field in &mut fields {
                let column = changed
                    .as_ref()
                    .filter(|column| column.name() == field.name)
                    .cloned()
                    .unwrap_or_else(|| {
                        storage
                            .children()
                            .into_iter()
                            .find(|column| column.name() == field.name)
                            .unwrap()
                    });
                let mut facets = column
                    .field()
                    .metadata()
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect::<std::collections::BTreeMap<_, _>>();
                facets.insert(
                    crate::codegen::boundary_projection::REGISTRY_FIELD_SOURCE.into(),
                    column.canonical_json().unwrap(),
                );
                field.named_facets = facets;
            }
            let mut graph = ProjectionGraph::default();
            graph
                .declare::<PresentedStorage>(
                    &definition.name,
                    definition.source,
                    definition.version,
                    definition.direction,
                    &definition.reserved_tags,
                    &definition
                        .reserved_names
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                    |_| Ok(Shape::Record(fields)),
                )
                .unwrap();
            graph
        };
        let imports = crate::codegen::boundary_projection::tests::imports();
        crate::codegen::boundary_projection::descriptors_with_imports(
            &graph(None),
            PACKAGE,
            "storage.proto",
            &imports,
        )
        .unwrap();
        let id = storage
            .children()
            .into_iter()
            .find(|column| column.name() == "quantity_type_id")
            .unwrap();
        for changed in [
            FieldContract::from_field(
                id.field()
                    .clone()
                    .with_data_type(DataType::FixedSizeBinary(32)),
            ),
            id.clone().optional(),
            FieldContract::from_field(id.field().clone().with_data_type(DataType::FixedSizeList(
                std::sync::Arc::new(arrow_schema::Field::new(
                    "item",
                    DataType::FixedSizeBinary(16),
                    false,
                )),
                2,
            ))),
        ] {
            assert!(
                crate::codegen::boundary_projection::descriptors_with_imports(
                    &graph(Some(changed)),
                    PACKAGE,
                    "storage.proto",
                    &imports
                )
                .is_err()
            );
        }
        assert!(
            types::projection_type(&id, "RawDomainId").is_err(),
            "ordinary source domain admission still requires an explicit semantic declaration"
        );
        assert!(
            types::check_native_contract(
                &id,
                "DeclaredId",
                Mode::Domain,
                &TypeRef::Scalar(native::Scalar::Bytes),
                PACKAGE
            )
            .is_err(),
            "opaque bytes are not a nominal identity representation"
        );
    }

    pse_contract::projection_struct! {
        struct NativeRecord {
            projection: { name: "NativeRecord", version: 1, direction: Direction::Both, reserved_tags: [7], reserved_names: ["retired"] };
            count: u64 => {tag: 9, direction: Direction::Both, rules: []};
        }
    }
    fn record() -> FieldContract {
        FieldContract::structure(vec![
            FieldContract::native(DataType::UInt64)
                .with_name("count")
                .with_boundary_tag(9),
        ])
        .with_boundary_shape(presentation(1))
    }
    fn presentation(version: u32) -> BoundaryShape {
        let mut shape = BoundaryShape::native(file!(), version, Direction::Both);
        shape.reserved_tags = vec![7];
        shape.reserved_names = vec!["retired".into()];
        shape
    }
    #[test]
    fn original_nested_record_cannot_hide_child_type_tag_or_local_rule_drift() {
        let original = record();
        checks::<NativeRecord>(&original).unwrap();
        for child in [
            FieldContract::native(DataType::Int64)
                .with_name("count")
                .with_boundary_tag(9),
            FieldContract::native(DataType::UInt64)
                .with_name("count")
                .with_boundary_tag(8),
            FieldContract::nonnegative(10)
                .with_name("count")
                .with_boundary_tag(9),
        ] {
            let changed = FieldContract::from_field(
                original
                    .field()
                    .clone()
                    .with_data_type(DataType::Struct(vec![child.into_field()].into())),
            );
            assert!(checks::<NativeRecord>(&changed).is_err());
        }
        let stale = original.with_boundary_shape(presentation(2));
        assert!(checks::<NativeRecord>(&stale).is_err());
    }

    struct NativeAlternatives;
    impl ProjectionType for NativeAlternatives {
        fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
            graph.declare::<Self>(
                "NativeAlternatives",
                file!(),
                1,
                Direction::Both,
                &[],
                &[],
                |graph| {
                    Ok(Shape::Alternatives(vec![
                        native::Alternative {
                            name: "absent".into(),
                            tag: 11,
                            fields: vec![],
                        },
                        native::Alternative {
                            name: "present".into(),
                            tag: 4,
                            fields: vec![native::Field::of::<NativePayload>(
                                graph,
                                "value",
                                1,
                                Direction::Both,
                                vec![LocalRule::Required],
                            )?],
                        },
                    ]))
                },
            )
        }
    }
    pse_contract::projection_struct! {
        struct NativePayload {
            projection: {name:"NativePayload", version:1, direction:Direction::Both, reserved_tags:[], reserved_names:[]};
            count:i64 => {tag:3, direction:Direction::Both, rules:[LocalRule::SignedRange {min:0,max:7}]};
        }
    }
    fn payload(maximum: i64) -> FieldContract {
        FieldContract::structure(vec![
            FieldContract::nonnegative(maximum)
                .with_name("count")
                .with_boundary_tag(3),
        ])
        .with_name("payload")
        .optional()
        .with_boundary_shape(BoundaryShape::native(file!(), 1, Direction::Both))
    }
    #[test]
    fn original_alternative_payload_retains_its_actual_native_rules() {
        let value = FieldContract::structure(vec![
            FieldContract::native(DataType::Utf8).with_name("kind"),
            payload(7),
        ])
        .with_alternative(
            &TaggedAlternative::new("kind", [("present".into(), "payload".into())])
                .with_unit("absent"),
        )
        .with_boundary_shape(
            BoundaryShape::native(file!(), 1, Direction::Both)
                .alternatives([("present".into(), 4), ("absent".into(), 11)]),
        );
        checks::<NativeAlternatives>(&value).unwrap();
        let changed = FieldContract::from_field(
            value.field().clone().with_data_type(DataType::Struct(
                vec![
                    FieldContract::native(DataType::Utf8)
                        .with_name("kind")
                        .into_field(),
                    payload(8).into_field(),
                ]
                .into(),
            )),
        );
        assert!(checks::<NativeAlternatives>(&changed).is_err());
    }
}
