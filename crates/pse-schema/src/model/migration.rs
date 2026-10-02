// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Schema migrations (blueprint §4.1 `reference.schema_migrations`, §20.5).
//!
//! An explicit schema transformation is declared here and compiled by the catalog
//! into native projections, checked defaults and nullability obligations. The caller
//! selects both versions through this declaration; store opening never discovers
//! or applies a conversion path.

use crate::literal::NativeLiteral;

/// Explicit treatment of absent source values. Partial composite keys never map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationNulls {
    /// Preserve an entirely absent tuple when every target field permits absence.
    Preserve,
    /// Refuse any absent source value.
    Reject,
}

/// One portable, typed source-to-target tuple.
#[derive(Clone, Debug, PartialEq)]
pub struct MigrationValuePair {
    /// Exact source literals, in selected column order.
    pub source: Vec<NativeLiteral>,
    /// Exact target literals, in selected column order.
    pub target: Vec<NativeLiteral>,
}

/// No implicit identity fallback exists for a finite mapping.
#[derive(Clone, Debug, PartialEq)]
pub enum MappingPolicy {
    /// Explicitly retain values, checking them against the target meaning.
    Identity,
    /// A complete finite mapping of the values encountered in the selected source.
    Finite(Vec<MigrationValuePair>),
}

/// Portable mapping policy shared by domain recoding and composite key mapping.
#[derive(Clone, Debug, PartialEq)]
pub struct MigrationValues {
    /// Declared transformation; unseen finite-map values refuse.
    pub policy: MappingPolicy,
    /// Absence policy, independently declared rather than inferred from a default.
    pub nulls: MigrationNulls,
}

impl MigrationValues {
    /// Validate the portable map against independently selected source and target fields.
    /// Values are subsequently checked by the native predicate compiler; key existence
    /// and reference closure are relation-wide obligations of the complete output.
    /// # Errors
    /// Wrong arity/fields, null literals, duplicate source keys or noninjective key maps.
    pub fn validate_fields(
        &self,
        source: &[arrow_schema::Field],
        target: &[arrow_schema::Field],
        injective: bool,
    ) -> Result<(), crate::SchemaError> {
        let invalid = |reason: &str| crate::SchemaError::InvalidDeclaration {
            context: "migration mapping".into(),
            reason: reason.into(),
        };
        if source.is_empty() || source.len() != target.len() {
            return Err(invalid(
                "mapping requires equal nonzero source and target arity",
            ));
        }
        if self.nulls == MigrationNulls::Preserve && target.iter().any(|field| !field.is_nullable())
        {
            return Err(invalid(
                "preserving absence requires nullable target fields",
            ));
        }
        if let MappingPolicy::Finite(rows) = &self.policy {
            if rows.is_empty() {
                return Err(invalid(
                    "finite mapping must explicitly declare at least one tuple",
                ));
            }
            let mut sources = Vec::new();
            let mut targets = Vec::new();
            for row in rows {
                for (values, fields) in [(&row.source, source), (&row.target, target)] {
                    if values.len() != fields.len() {
                        return Err(invalid("mapping tuple arity differs"));
                    }
                    for (value, field) in values.iter().zip(fields) {
                        if pse_columnar::native_field::project(
                            value.field(),
                            pse_columnar::native_field::MetadataPurpose::ValueIdentity,
                        )
                        .map_err(|error| invalid(&error.to_string()))?
                            != pse_columnar::native_field::project(
                                field,
                                pse_columnar::native_field::MetadataPurpose::ValueIdentity,
                            )
                            .map_err(|error| invalid(&error.to_string()))?
                            || value.field().data_type() != field.data_type()
                            || value.field().is_nullable() != field.is_nullable()
                        {
                            return Err(invalid(
                                "mapping literal does not carry its exact selected field",
                            ));
                        }
                        if value.array().null_count() != 0 {
                            return Err(invalid(
                                "null map literals are forbidden; declare an absence policy",
                            ));
                        }
                    }
                }
                let source = row
                    .source
                    .iter()
                    .map(NativeLiteral::scalar)
                    .collect::<Result<Vec<_>, _>>()?;
                let target = row
                    .target
                    .iter()
                    .map(NativeLiteral::scalar)
                    .collect::<Result<Vec<_>, _>>()?;
                if sources.contains(&source) {
                    return Err(invalid("mapping source tuple is duplicated"));
                }
                if targets.contains(&target) && injective {
                    return Err(invalid("reference-key mapping is not injective"));
                }
                sources.push(source);
                targets.push(target);
            }
        }
        Ok(())
    }

    fn as_json(&self) -> serde_json::Value {
        let policy = match &self.policy {
            MappingPolicy::Identity => serde_json::json!(["identity"]),
            MappingPolicy::Finite(rows) => serde_json::json!([
                "finite",
                rows.iter()
                    .map(|row| {
                        serde_json::json!([
                            row.source
                                .iter()
                                .map(NativeLiteral::as_json)
                                .collect::<Vec<_>>(),
                            row.target
                                .iter()
                                .map(NativeLiteral::as_json)
                                .collect::<Vec<_>>()
                        ])
                    })
                    .collect::<Vec<_>>()
            ]),
        };
        serde_json::json!({"policy": policy, "nulls": match self.nulls { MigrationNulls::Preserve => "preserve", MigrationNulls::Reject => "reject" }})
    }
}

/// One step of a migration.
#[derive(Clone, Debug, PartialEq)]
pub enum MigrationStep {
    /// Add a column, filling existing rows with `default`.
    AddColumn {
        /// The new column's name, which must exist in the target version.
        name: &'static str,
        /// The value every existing row gets.
        default: NativeLiteral,
    },
    /// Drop a column.
    DropColumn(&'static str),
    /// Rename a column, keeping its values.
    RenameColumn {
        /// The name in the source version.
        from: &'static str,
        /// The name in the target version.
        to: &'static str,
    },
    /// Widen or narrow a column's nullability.
    ChangeNullable {
        /// The column.
        name: &'static str,
        /// The nullability in the target version.
        nullable: bool,
    },
    /// Recode one domain through exact typed values or declared identity policy.
    RecodeDomain {
        /// Source and target column name at this ordered step.
        name: &'static str,
        /// Explicit finite or identity policy.
        mapping: MigrationValues,
    },
    /// Map a composite reference key as a correlated tuple, never independently.
    MapReferenceKey {
        /// Selected key columns, in declared order.
        columns: Vec<&'static str>,
        /// Source keys must be unique; finite target tuples must also be unique.
        mapping: MigrationValues,
    },
}

impl MigrationStep {
    /// The `MigrationOp` enumeration member this step is.
    pub const fn op(&self) -> &'static str {
        match self {
            Self::AddColumn { .. } => "add_column",
            Self::DropColumn(_) => "drop_column",
            Self::RenameColumn { .. } => "rename_column",
            Self::ChangeNullable { .. } => "change_nullable",
            Self::RecodeDomain { .. } => "recode_domain",
            Self::MapReferenceKey { .. } => "map_reference_key",
        }
    }
}

/// A declared migration between exact source/target meanings of one relation.
/// Registered structural migrations advance versions. Independently admitted
/// recorded migrations may retain a version when support meaning changed and an
/// explicit value policy establishes the target (blueprint §4.1).
#[derive(Clone, Debug, PartialEq)]
pub struct MigrationSpec {
    /// The qualified relation, for example `authored.stoichiometry`.
    pub relation: &'static str,
    /// The version the migration reads.
    pub from_version: u32,
    /// The version the migration writes.
    pub to_version: u32,
    /// The steps, in application order.
    pub steps: Vec<MigrationStep>,
    /// Why the schema changed.
    pub doc: &'static str,
}

impl MigrationSpec {
    /// The migration's registry name, `<relation>@<from>-><to>`.
    pub fn qualified_name(&self) -> String {
        format!(
            "{}@{}->{}",
            self.relation, self.from_version, self.to_version
        )
    }

    /// The `plan_spec` rendering stored in `reference.schema_migrations`.
    ///
    /// One line per step, in application order: a plan a reader can check against the two
    /// schemas, not a prose summary of one.
    pub fn plan_spec(&self) -> String {
        let mut rendered = Vec::with_capacity(self.steps.len());
        for step in &self.steps {
            rendered.push(match step {
                MigrationStep::AddColumn { name, default } => format!(
                    "add_column {} {}",
                    serde_json::json!(["text", name]),
                    default.as_json()
                ),
                MigrationStep::DropColumn(name) => {
                    format!("drop_column {}", serde_json::json!(["text", name]))
                }
                MigrationStep::RenameColumn { from, to } => format!(
                    "rename_column {} {}",
                    serde_json::json!(["text", from]),
                    serde_json::json!(["text", to])
                ),
                MigrationStep::ChangeNullable { name, nullable } => {
                    format!(
                        "change_nullable {} {nullable}",
                        serde_json::json!(["text", name])
                    )
                }
                MigrationStep::RecodeDomain { name, mapping } => format!(
                    "recode_domain {} {}",
                    serde_json::json!(["text", name]),
                    mapping.as_json()
                ),
                MigrationStep::MapReferenceKey { columns, mapping } => format!(
                    "map_reference_key {} {}",
                    serde_json::json!(columns),
                    mapping.as_json()
                ),
            });
        }
        rendered.join("\n")
    }
}

#[cfg(test)]
mod mapping_unit {
    use super::*;
    use arrow_schema::{DataType, Field};
    use std::sync::Arc;

    fn literal(field: &Field, value: i64) -> NativeLiteral {
        NativeLiteral::from_scalar(
            Arc::new(field.clone()),
            &datafusion_common::ScalarValue::Int64(Some(value)),
        )
        .unwrap()
    }
    #[test]
    fn finite_keys_refuse_duplicates_nulls_wrong_fields_and_noninjectivity() {
        let source = Field::new("key", DataType::Int64, false);
        let target = source.clone();
        let row = |from, to| MigrationValuePair {
            source: vec![literal(&source, from)],
            target: vec![literal(&target, to)],
        };
        let mapping = |rows| MigrationValues {
            policy: MappingPolicy::Finite(rows),
            nulls: MigrationNulls::Reject,
        };
        assert!(
            mapping(vec![row(1, 2)])
                .validate_fields(&[source.clone()], &[target.clone()], true)
                .is_ok()
        );
        assert!(
            mapping(vec![row(1, 2), row(1, 3)])
                .validate_fields(&[source.clone()], &[target.clone()], false)
                .is_err()
        );
        assert!(
            mapping(vec![row(1, 2), row(3, 2)])
                .validate_fields(&[source.clone()], &[target.clone()], true)
                .is_err()
        );
        assert!(
            mapping(vec![row(1, 2), row(3, 2)])
                .validate_fields(&[source.clone()], &[target.clone()], false)
                .is_ok()
        );
        let wrong = Field::new("key", DataType::UInt64, false);
        assert!(
            mapping(vec![row(1, 2)])
                .validate_fields(&[source.clone()], &[wrong], true)
                .is_err()
        );
        let nullable = source.clone().with_nullable(true);
        let null = NativeLiteral::from_scalar(
            Arc::new(nullable.clone()),
            &datafusion_common::ScalarValue::Int64(None),
        )
        .unwrap();
        assert!(
            mapping(vec![MigrationValuePair {
                source: vec![null],
                target: vec![literal(&target, 2)]
            }])
            .validate_fields(&[nullable], &[target], true)
            .is_err()
        );
    }
    #[test]
    fn identity_and_absence_are_explicit_and_portable_plan_changes_with_values() {
        let field = Field::new("key", DataType::Int64, false);
        let identity = MigrationValues {
            policy: MappingPolicy::Identity,
            nulls: MigrationNulls::Preserve,
        };
        assert!(
            identity
                .validate_fields(&[field.clone()], &[field.clone()], true)
                .is_err()
        );
        assert!(
            identity
                .validate_fields(
                    &[field.clone().with_nullable(true)],
                    &[field.clone().with_nullable(true)],
                    true
                )
                .is_ok()
        );
        let spec = |value| MigrationSpec {
            relation: "authored.keys",
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep::MapReferenceKey {
                columns: vec!["key"],
                mapping: MigrationValues {
                    policy: MappingPolicy::Finite(vec![MigrationValuePair {
                        source: vec![literal(&field, 1)],
                        target: vec![literal(&field, value)],
                    }]),
                    nulls: MigrationNulls::Reject,
                },
            }],
            doc: "map",
        };
        assert_ne!(spec(2).plan_spec(), spec(3).plan_spec());
        assert!(spec(2).plan_spec().contains("map_reference_key"));
    }
}
