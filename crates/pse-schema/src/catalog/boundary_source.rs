// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Apply source-owned wire numbers to actual native declarations without restating types.

use std::collections::BTreeSet;

use crate::SchemaError;
use crate::model::{BoundaryRecord, BoundaryShape, FieldContract, RelationDecl};
use arrow_schema::{DataType, Field};

/// Only presentation lives here: exact native field names and explicit wire numbers.
pub(super) struct RecordLayout {
    pub path: &'static str,
    pub fields: &'static [(&'static str, u32)],
    pub alternatives: &'static [(&'static str, u32)],
}

/// Publish an actual selected native row, refusing drift from its source wire layout.
pub(super) fn present(
    mut declaration: RelationDecl,
    source: &'static str,
    name: &'static str,
    layouts: &[RecordLayout],
) -> Result<RelationDecl, SchemaError> {
    let version = declaration.key.version;
    let direction = pse_contract::BoundaryDirection::Both;
    let mut used = BTreeSet::new();
    declaration.columns = fields(declaration.columns, "", source, version, layouts, &mut used)?;
    if used.len() != layouts.len() {
        return Err(crate::checks::invalid(
            source,
            "wire layout contains an absent or repeated native record path",
        ));
    }
    Ok(declaration.with_boundary_record(BoundaryRecord {
        source,
        name,
        version,
        direction,
        reserved_tags: &[],
        reserved_names: &[],
    }))
}

fn layout<'a>(layouts: &'a [RecordLayout], path: &str) -> Result<&'a RecordLayout, SchemaError> {
    let mut matches = layouts.iter().filter(|layout| layout.path == path);
    let found = matches
        .next()
        .ok_or_else(|| crate::checks::invalid(path, "native record has no source wire layout"))?;
    if matches.next().is_some() {
        return Err(crate::checks::invalid(
            path,
            "native record has duplicate source wire layouts",
        ));
    }
    Ok(found)
}

fn fields(
    fields: Vec<FieldContract>,
    path: &str,
    source: &str,
    version: u32,
    layouts: &[RecordLayout],
    used: &mut BTreeSet<String>,
) -> Result<Vec<FieldContract>, SchemaError> {
    let layout = layout(layouts, path)?;
    used.insert(path.into());
    let names = fields
        .iter()
        .map(FieldContract::name)
        .collect::<BTreeSet<_>>();
    let declared = layout
        .fields
        .iter()
        .map(|(name, _)| *name)
        .collect::<BTreeSet<_>>();
    if names != declared || declared.len() != layout.fields.len() || names.len() != fields.len() {
        return Err(crate::checks::invalid(
            format!("{source}#{path}"),
            "wire field layout must cover exactly the actual native field names once",
        ));
    }
    fields
        .into_iter()
        .map(|field| {
            let tag = layout
                .fields
                .iter()
                .find_map(|(name, number)| (*name == field.name()).then_some(*number))
                .ok_or_else(|| {
                    crate::checks::invalid(path, "native field has no explicit wire number")
                })?;
            let child_path = if path.is_empty() {
                field.name().into()
            } else {
                format!("{path}.{}", field.name())
            };
            annotate(
                field.with_boundary_tag(tag),
                &child_path,
                source,
                version,
                layouts,
                used,
            )
        })
        .collect()
}

fn annotate(
    field: FieldContract,
    path: &str,
    source: &str,
    version: u32,
    layouts: &[RecordLayout],
    used: &mut BTreeSet<String>,
) -> Result<FieldContract, SchemaError> {
    // A shared factory may already declare its complete presentation inline.
    if BoundaryShape::from_field(field.field())?.is_some() {
        return Ok(field);
    }
    let mut native = field.into_field();
    let data_type = match native.data_type().clone() {
        DataType::Struct(children) => {
            let path = FieldContract::from_field(native.clone())
                .structure_name()
                .map_or_else(|| path.to_owned(), |name| format!("@{name}"));
            let children = fields(
                children
                    .iter()
                    .map(|child| FieldContract::from_field(child.as_ref().clone()))
                    .collect(),
                &path,
                source,
                version,
                layouts,
                used,
            )?;
            native = native.with_data_type(DataType::Struct(
                children
                    .into_iter()
                    .map(FieldContract::into_field)
                    .collect(),
            ));
            let presentation = layout(layouts, &path)?;
            let shape =
                BoundaryShape::native(source, version, pse_contract::BoundaryDirection::Both)
                    .alternatives(
                        presentation
                            .alternatives
                            .iter()
                            .map(|(name, tag)| ((*name).into(), *tag)),
                    );
            let field = FieldContract::from_field(native).with_boundary_shape(shape);
            BoundaryShape::from_field(field.field())?;
            return Ok(field);
        }
        DataType::List(child) => {
            DataType::List(annotate_child(&child, path, source, version, layouts, used)?.into())
        }
        DataType::LargeList(child) => DataType::LargeList(
            annotate_child(&child, path, source, version, layouts, used)?.into(),
        ),
        DataType::FixedSizeList(child, size) => DataType::FixedSizeList(
            annotate_child(&child, path, source, version, layouts, used)?.into(),
            size,
        ),
        data_type => data_type,
    };
    Ok(FieldContract::from_field(native.with_data_type(data_type)))
}

fn annotate_child(
    child: &Field,
    path: &str,
    source: &str,
    version: u32,
    layouts: &[RecordLayout],
    used: &mut BTreeSet<String>,
) -> Result<Field, SchemaError> {
    annotate(
        FieldContract::from_field(child.clone()),
        path,
        source,
        version,
        layouts,
        used,
    )
    .map(FieldContract::into_field)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Authority, Namespace, SnapshotClass, TaggedAlternative};

    const ROOT: RecordLayout = RecordLayout {
        path: "",
        fields: &[("id", 37), ("payload", 8)],
        alternatives: &[],
    };
    const PAYLOAD: RecordLayout = RecordLayout {
        path: "payload",
        fields: &[("x", 11), ("y", 4)],
        alternatives: &[],
    };
    fn declaration() -> RelationDecl {
        RelationDecl::new(
            Namespace::Authored,
            "wire_source",
            3,
            Authority::Authored,
            SnapshotClass::Model,
            "actual source",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::id(), "actual identity"),
            FieldContract::structure(vec![
                FieldContract::native(DataType::Float64)
                    .with_name("x")
                    .with_quantity("temperature")
                    .with_doc("physical meaning"),
                FieldContract::list(FieldContract::nonnegative(17))
                    .with_name("y")
                    .optional(),
            ])
            .with_name("payload"),
        ])
    }
    fn strip(field: FieldContract) -> FieldContract {
        let mut native = field.into_field();
        native
            .metadata_mut()
            .remove(crate::model::field_facets::BOUNDARY_TAG);
        native
            .metadata_mut()
            .remove(crate::model::field_facets::BOUNDARY_SHAPE);
        let data_type = match native.data_type().clone() {
            DataType::Struct(children) => DataType::Struct(
                children
                    .iter()
                    .map(|child| {
                        strip(FieldContract::from_field(child.as_ref().clone())).into_field()
                    })
                    .collect(),
            ),
            DataType::List(child) => DataType::List(
                strip(FieldContract::from_field(child.as_ref().clone()))
                    .into_field()
                    .into(),
            ),
            other => other,
        };
        FieldContract::from_field(native.with_data_type(data_type))
    }

    #[test]
    fn native_tree_and_scientific_facets_survive_explicit_wire_presentation() {
        let original = declaration();
        let projected =
            present(original.clone(), file!(), "ActualSource", &[ROOT, PAYLOAD]).unwrap();
        assert_eq!(
            projected.boundary.unwrap().direction,
            pse_contract::BoundaryDirection::Both
        );
        assert_eq!(
            projected
                .columns
                .clone()
                .into_iter()
                .map(strip)
                .collect::<Vec<_>>(),
            original.columns
        );
        let children = projected.columns[1].children();
        assert_eq!(children[0].boundary_tag(), Some(11));
        assert_eq!(children[1].boundary_tag(), Some(4));
        // Physical order is a separate native contract, never the wire-number authority.
        let mut reordered = original;
        reordered.columns.reverse();
        let reordered = present(reordered, file!(), "ActualSource", &[ROOT, PAYLOAD]).unwrap();
        assert_eq!(reordered.columns[0].boundary_tag(), Some(8));
        assert_eq!(reordered.columns[1].boundary_tag(), Some(37));
    }

    #[test]
    fn field_layout_refuses_added_removed_duplicated_and_unreachable_members() {
        for fields in [
            &[("x", 11)][..],
            &[("x", 11), ("y", 4), ("absent", 12)][..],
            &[("x", 11), ("y", 4), ("y", 15)][..],
            &[("x", 11), ("y", 11)][..],
        ] {
            let layouts = [
                ROOT,
                RecordLayout {
                    path: "payload",
                    fields,
                    alternatives: &[],
                },
            ];
            assert!(present(declaration(), file!(), "ActualSource", &layouts).is_err());
        }
        let layouts = [
            ROOT,
            PAYLOAD,
            RecordLayout {
                path: "absent",
                fields: &[],
                alternatives: &[],
            },
        ];
        assert!(present(declaration(), file!(), "ActualSource", &layouts).is_err());
        let layouts = [ROOT, PAYLOAD, PAYLOAD];
        assert!(present(declaration(), file!(), "ActualSource", &layouts).is_err());
    }

    #[test]
    fn native_alternative_population_requires_complete_nonoverlapping_explicit_tags() {
        let mut declaration = declaration();
        declaration.columns[1] = FieldContract::structure(vec![
            FieldContract::native(DataType::Utf8).with_name("kind"),
            FieldContract::structure(vec![
                FieldContract::native(DataType::Int64).with_name("value"),
            ])
            .with_name("number")
            .optional(),
        ])
        .with_name("payload")
        .with_alternative(
            &TaggedAlternative::new("kind", [("number".into(), "number".into())])
                .with_unit("missing"),
        );
        let apply = |alternatives: &'static [(&'static str, u32)]| {
            present(
                declaration.clone(),
                file!(),
                "ActualSource",
                &[
                    ROOT,
                    RecordLayout {
                        path: "payload",
                        fields: &[("kind", 9), ("number", 3)],
                        alternatives,
                    },
                    RecordLayout {
                        path: "payload.number",
                        fields: &[("value", 41)],
                        alternatives: &[],
                    },
                ],
            )
        };
        let valid = apply(&[("number", 71), ("missing", 8)]).unwrap();
        let shape = BoundaryShape::from_field(valid.columns[1].field())
            .unwrap()
            .unwrap();
        assert_eq!(
            shape.alternatives,
            [("missing".into(), 8), ("number".into(), 71)]
        );
        for alternatives in [
            &[("number", 71)][..],
            &[("number", 71), ("missing", 8), ("extra", 9)][..],
            &[("number", 71), ("missing", 71)][..],
            &[("number", 71), ("missing", 8), ("missing", 9)][..],
            &[("number", 0), ("missing", 8)][..],
        ] {
            assert!(apply(alternatives).is_err());
        }
    }

    #[test]
    fn selected_actual_catalog_closure_has_native_shapes_and_stable_numbers() {
        let registry = crate::catalog::assemble().unwrap();
        for (relation, native, version) in [
            (
                "modeling_declarations",
                "AuthoredModelingDeclarationsRow",
                27,
            ),
            ("documents", "AuthoredDocumentsRow", 2),
            ("fit_cases", "AuthoredFitCasesRow", 4),
            (
                "numerical_requirements",
                "AuthoredNumericalRequirementsRow",
                3,
            ),
            ("accuracy_goals", "AuthoredAccuracyGoalsRow", 1),
            ("caloric_requirements", "AuthoredCaloricRequirementsRow", 1),
            ("engineering_scales", "AuthoredEngineeringScalesRow", 1),
            (
                "engineering_default_rules",
                "AuthoredEngineeringDefaultRulesRow",
                1,
            ),
        ] {
            let row = registry.relation(&format!("authored.{relation}")).unwrap();
            let boundary = row.boundary.unwrap();
            assert_eq!(
                (boundary.name, boundary.version, boundary.direction),
                (native, version, pse_contract::BoundaryDirection::Both)
            );
            let mut fields = Vec::new();
            for field in &row.columns {
                field.walk(&mut fields);
            }
            for field in fields {
                if matches!(field.data_type(), DataType::Struct(_)) && field.extension().is_none() {
                    let shape = BoundaryShape::from_field(field.field()).unwrap().unwrap();
                    assert_eq!(shape.direction, pse_contract::BoundaryDirection::Both);
                }
            }
        }
        let row = registry.relation("authored.modeling_declarations").unwrap();
        assert_eq!(row.column("source_start").unwrap().boundary_tag(), Some(7));
        assert_eq!(row.column("value").unwrap().boundary_tag(), Some(9));
        let shape = BoundaryShape::from_field(row.column("value").unwrap().field())
            .unwrap()
            .unwrap();
        for (tag, number) in [
            ("package", 4),
            ("variable", 16),
            ("reference_translation", 29),
            ("inventory_balance", 56),
            ("identifier_scheme", 66),
        ] {
            assert!(shape.alternatives.contains(&(tag.into(), number)));
        }
    }
}
