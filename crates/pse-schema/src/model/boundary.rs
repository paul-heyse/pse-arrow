// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Wire presentation declared beside a registry relation; scientific fields remain authoritative.

/// Explicit publication of one resolved relation through an owner-derived record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundaryRecord {
    /// Actual declaration source, captured at the source owner.
    pub source: &'static str,
    /// Stable message name, independent of field order or native storage spelling.
    pub name: &'static str,
    /// Scientific interpretation version, separate from the enclosing wire package major.
    pub version: u32,
    /// Public consumer direction.
    pub direction: pse_contract::BoundaryDirection,
    /// Retired field numbers, never available for reuse.
    pub reserved_tags: &'static [u32],
    /// Retired public field names, never available for reuse.
    pub reserved_names: &'static [&'static str],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};

    fn declaration() -> RelationDecl {
        RelationDecl::new(
            Namespace::Reference,
            "selected_boundary",
            1,
            Authority::Reference,
            SnapshotClass::Model,
            "selected projection",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::id().with_name("id").with_boundary_tag(7),
        ])
        .with_boundary_record(BoundaryRecord {
            source: file!(),
            name: "SelectedBoundary",
            version: 1,
            direction: pse_contract::BoundaryDirection::Output,
            reserved_tags: &[2],
            reserved_names: &["removed"],
        })
    }

    #[test]
    fn selected_boundary_numbers_are_explicit_and_never_recycled() {
        let valid = declaration();
        crate::checks::relation_declaration(&valid).unwrap();
        for number in [0, 2, 19_000, 536_870_912] {
            let mut invalid = valid.clone();
            invalid.columns[0] = invalid.columns[0].clone().with_boundary_tag(number);
            assert!(crate::checks::relation_declaration(&invalid).is_err());
        }
        let mut duplicate = valid.clone();
        duplicate.columns.push(
            FieldContract::native(arrow_schema::DataType::Utf8)
                .with_name("other")
                .with_boundary_tag(7),
        );
        assert!(crate::checks::relation_declaration(&duplicate).is_err());
        let mut absent = valid.clone();
        absent.columns[0] = FieldContract::id().with_name("id");
        assert!(crate::checks::relation_declaration(&absent).is_err());
        let mut renamed = valid;
        renamed.columns[0] = renamed.columns[0].clone().with_name("removed");
        renamed.primary_key = Some(vec!["removed"]);
        assert!(crate::checks::relation_declaration(&renamed).is_err());
    }

    #[test]
    fn wire_numbers_preserve_the_original_value_domain() {
        let original = FieldContract::id().with_name("original");
        assert_eq!(
            original.value_type(),
            original.clone().with_boundary_tag(7).value_type()
        );
        assert_ne!(
            original.field(),
            original.clone().with_boundary_tag(7).field(),
            "the actual source still retains its distinct wire declaration"
        );
    }
}
