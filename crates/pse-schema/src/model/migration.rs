// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Schema migrations (blueprint §4.1 `reference.schema_migrations`, §20.5).
//!
//! A migration is declared, not written: `pse-relations` generates
//! `migrate_v1_to_v2(batch)` from these steps, so an artifact written under an older
//! contract is read by code that knows exactly what changed rather than by code that
//! guesses from the schema it finds.

use crate::model::cell::Cell;

/// One step of a migration.
#[derive(Clone, Debug, PartialEq)]
pub enum MigrationStep {
    /// Add a column, filling existing rows with `default`.
    AddColumn {
        /// The new column's name, which must exist in the target version.
        name: &'static str,
        /// The value every existing row gets.
        default: Cell,
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
}

impl MigrationStep {
    /// The `MigrationOp` enumeration member this step is.
    pub const fn op(&self) -> &'static str {
        match self {
            Self::AddColumn { .. } => "add_column",
            Self::DropColumn(_) => "drop_column",
            Self::RenameColumn { .. } => "rename_column",
            Self::ChangeNullable { .. } => "change_nullable",
        }
    }
}

/// A declared migration between two versions of one relation (blueprint §4.1).
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
                    Cell::text(*name).literal_spec(),
                    default.literal_spec()
                ),
                MigrationStep::DropColumn(name) => {
                    format!("drop_column {}", Cell::text(*name).literal_spec())
                }
                MigrationStep::RenameColumn { from, to } => format!(
                    "rename_column {} {}",
                    Cell::text(*from).literal_spec(),
                    Cell::text(*to).literal_spec()
                ),
                MigrationStep::ChangeNullable { name, nullable } => {
                    format!(
                        "change_nullable {} {nullable}",
                        Cell::text(*name).literal_spec()
                    )
                }
            });
        }
        rendered.join("\n")
    }
}
