// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit wire presentation for an actual nested native field declaration.

use super::{BoundaryRecord, FieldContract, TaggedAlternative};
use crate::SchemaError;
use std::collections::{BTreeMap, BTreeSet};

/// Co-located presentation metadata. Field and payload types stay on the actual
/// native field; this contains only names, interpretation and explicit numbers.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryShape {
    /// Source owner of this presentation declaration.
    pub source: String,
    /// Stable explicit message name; absence selects the actual generated native
    /// type's source name, never a field ordinal or inferred physical shape.
    pub name: Option<String>,
    /// Scientific interpretation of this shape, separate from package version.
    pub version: u32,
    /// Actual public consumer direction.
    pub direction: pse_vocabulary::BoundaryDirection,
    /// Retired numbers, unavailable to fields or alternatives.
    pub reserved_tags: Vec<u32>,
    /// Retired public names.
    pub reserved_names: Vec<String>,
    /// Explicit alternative number for every actual native tag. Empty for records.
    pub alternatives: Vec<(String, u32)>,
}

impl BoundaryShape {
    /// Publish a native record's shape without declaring its fields a second time.
    pub fn record(record: BoundaryRecord) -> Self {
        Self {
            source: record.source.into(),
            name: Some(record.name.into()),
            version: record.version,
            direction: record.direction,
            reserved_tags: record.reserved_tags.to_vec(),
            reserved_names: record
                .reserved_names
                .iter()
                .map(|name| (*name).into())
                .collect(),
            alternatives: Vec::new(),
        }
    }
    /// Project an anonymous native source type under that actual type's name.
    pub fn native(
        source: &str,
        version: u32,
        direction: pse_vocabulary::BoundaryDirection,
    ) -> Self {
        Self {
            source: source.into(),
            name: None,
            version,
            direction,
            reserved_tags: Vec::new(),
            reserved_names: Vec::new(),
            alternatives: Vec::new(),
        }
    }
    /// Add explicit wire alternative numbers. Registry admission checks exact
    /// membership against the actual native discriminator declaration.
    #[must_use]
    pub fn alternatives(mut self, alternatives: impl IntoIterator<Item = (String, u32)>) -> Self {
        self.alternatives = alternatives.into_iter().collect();
        self.alternatives.sort();
        self
    }
    /// Canonical facet encoding, preserving one source of wire presentation.
    pub fn canonical(&self) -> String {
        // Every member is serializable by construction; constructing Value directly
        // also avoids a fallible formatting route on the declaration builder.
        let mut value = serde_json::json!({
            "source":self.source,"name":self.name,"version":self.version,
            "direction":self.direction,"reserved_tags":self.reserved_tags,
            "reserved_names":self.reserved_names,"alternatives":self.alternatives,
        });
        value.sort_all_objects();
        value.to_string()
    }
    /// Decode and check presentation against its actual native structure.
    /// # Errors
    /// Malformed metadata, missing/extra alternatives or absent/recycled numbers.
    pub fn from_field(native: &arrow_schema::Field) -> Result<Option<Self>, SchemaError> {
        let field = FieldContract::from_field(native.clone());
        let Some(text) = native.metadata().get(super::field_facets::BOUNDARY_SHAPE) else {
            return Ok(None);
        };
        let invalid = |reason: &str| crate::checks::invalid(field.name(), reason);
        let shape: Self = serde_json::from_str(text)
            .map_err(|_| invalid("malformed nested boundary presentation"))?;
        if shape.canonical() != *text
            || shape.source.is_empty()
            || shape.version == 0
            || shape.name.as_ref().is_some_and(|name| !identifier(name))
        {
            return Err(invalid(
                "nested presentation requires canonical metadata, source, name and interpretation",
            ));
        }
        if !matches!(field.data_type(), arrow_schema::DataType::Struct(_)) {
            return Err(invalid(
                "nested record/alternative presentation requires the actual native structure",
            ));
        }
        let reserved = shape.reserved_tags.iter().copied().collect::<BTreeSet<_>>();
        let names = shape
            .reserved_names
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if reserved.len() != shape.reserved_tags.len()
            || names.len() != shape.reserved_names.len()
            || reserved.iter().any(|tag| !valid_tag(*tag))
            || names.iter().any(|name| !identifier(name))
        {
            return Err(invalid(
                "nested presentation has invalid or duplicate reservations",
            ));
        }
        let alternative = TaggedAlternative::from_field(native)?;
        let mut tags = BTreeSet::new();
        if let Some(alternative) = alternative {
            let alternatives = shape
                .alternatives
                .iter()
                .map(|(name, tag)| (name.as_str(), *tag))
                .collect::<BTreeMap<_, _>>();
            if alternatives.len() != shape.alternatives.len()
                || alternative.arms.len() != alternatives.len()
                || alternative
                    .arms
                    .keys()
                    .any(|name| !alternatives.contains_key(name.as_str()))
            {
                return Err(invalid(
                    "wire alternatives must cover exactly the actual native tag population",
                ));
            }
            for (name, tag) in &shape.alternatives {
                if !identifier(name)
                    || !valid_tag(*tag)
                    || !tags.insert(*tag)
                    || reserved.contains(tag)
                    || names.contains(name.as_str())
                {
                    return Err(invalid(
                        "wire alternative has an invalid, duplicate or reserved name/number",
                    ));
                }
            }
        } else {
            if !shape.alternatives.is_empty() {
                return Err(invalid(
                    "record presentation cannot invent native alternatives",
                ));
            }
            for child in field.children() {
                let tag = child.boundary_tag().ok_or_else(|| {
                    invalid("nested record child needs an explicit source-owned field number")
                })?;
                if !valid_tag(tag)
                    || !tags.insert(tag)
                    || reserved.contains(&tag)
                    || names.contains(child.name())
                {
                    return Err(invalid(
                        "nested record child has an invalid, duplicate or reserved name/number",
                    ));
                }
            }
        }
        Ok(Some(shape))
    }
}
fn identifier(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphabetic() || byte == b'_' || (index > 0 && byte.is_ascii_digit())
        })
}
fn valid_tag(tag: u32) -> bool {
    tag > 0 && tag <= 536_870_911 && !(19_000..=19_999).contains(&tag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_schema::DataType;
    use pse_vocabulary::BoundaryDirection;

    #[test]
    fn wire_presentation_preserves_execution_value_and_logical_type_identity() {
        use pse_columnar::native_field::{MetadataPurpose, project};
        let plain = FieldContract::structure(vec![
            FieldContract::native(DataType::Int64).with_name("value"),
        ])
        .with_name("original");
        let wire = FieldContract::structure(vec![
            FieldContract::native(DataType::Int64)
                .with_name("value")
                .with_boundary_tag(17),
        ])
        .with_name("original")
        .with_boundary_shape(BoundaryShape::native(file!(), 3, BoundaryDirection::Both));
        for purpose in [
            MetadataPurpose::ExecutionIdentity,
            MetadataPurpose::ValueIdentity,
            MetadataPurpose::LogicalTypeIdentity,
        ] {
            assert_eq!(
                project(plain.field(), purpose).unwrap(),
                project(wire.field(), purpose).unwrap()
            );
        }
        assert_ne!(
            project(plain.field(), MetadataPurpose::PhysicalObservation).unwrap(),
            project(wire.field(), MetadataPurpose::PhysicalObservation).unwrap()
        );
    }

    fn record() -> FieldContract {
        FieldContract::structure(vec![
            FieldContract::native(DataType::Utf8)
                .with_name("label")
                .with_boundary_tag(7),
            FieldContract::native(DataType::Int64)
                .with_name("count")
                .with_boundary_tag(13),
        ])
        .with_name("original")
        .with_boundary_shape(BoundaryShape::native(file!(), 1, BoundaryDirection::Input))
    }

    #[test]
    fn nested_presentation_requires_actual_complete_stable_field_numbers() {
        let original = record();
        let shape = BoundaryShape::from_field(original.field())
            .unwrap()
            .unwrap();
        assert!(
            shape.name.is_none(),
            "anonymous presentation names its actual generated source type"
        );
        for tag in [0, 7, 19_000, 536_870_912] {
            let invalid = FieldContract::structure(vec![
                FieldContract::native(DataType::Utf8)
                    .with_name("label")
                    .with_boundary_tag(7),
                FieldContract::native(DataType::Int64)
                    .with_name("count")
                    .with_boundary_tag(tag),
            ])
            .with_boundary_shape(shape.clone());
            assert!(BoundaryShape::from_field(invalid.field()).is_err());
        }
        let missing = FieldContract::structure(vec![FieldContract::id().with_name("new_field")])
            .with_boundary_shape(shape.clone());
        assert!(BoundaryShape::from_field(missing.field()).is_err());
        let scalar = FieldContract::id().with_boundary_shape(shape);
        assert!(BoundaryShape::from_field(scalar.field()).is_err());
        let schema = arrow_schema::Schema::new(vec![scalar.into_field()]);
        assert!(
            crate::field_contract::declaration(&schema).is_err(),
            "known metadata cannot hide on an unselected scalar"
        );
    }

    #[test]
    fn native_alternative_presentation_refuses_invented_missing_and_duplicate_arms() {
        let native = FieldContract::structure(vec![
            FieldContract::native(DataType::Utf8).with_name("kind"),
            FieldContract::structure(vec![FieldContract::id().with_name("value")])
                .with_name("payload")
                .optional(),
        ])
        .with_alternative(
            &TaggedAlternative::new("kind", [("present".into(), "payload".into())])
                .with_unit("absent"),
        );
        let shape = BoundaryShape::native(file!(), 1, BoundaryDirection::Input)
            .alternatives([("present".into(), 4), ("absent".into(), 11)]);
        assert!(
            BoundaryShape::from_field(native.clone().with_boundary_shape(shape.clone()).field())
                .is_ok()
        );
        for arms in [
            vec![("present".into(), 4)],
            vec![("present".into(), 4), ("invented".into(), 11)],
            vec![("present".into(), 4), ("absent".into(), 4)],
            vec![
                ("present".into(), 4),
                ("absent".into(), 11),
                ("present".into(), 12),
            ],
        ] {
            assert!(
                BoundaryShape::from_field(
                    native
                        .clone()
                        .with_boundary_shape(shape.clone().alternatives(arms))
                        .field()
                )
                .is_err()
            );
        }
    }
}
