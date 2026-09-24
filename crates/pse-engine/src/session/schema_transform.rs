// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit schema evolution compiles declared edits into the common native plan.

use super::{EngineSession, PreparedComputation, engine_session::engine, output, scalar};
use crate::EngineError;
use datafusion::logical_expr::{Expr, LogicalPlanBuilder};
use pse_columnar::CancellationToken;
use pse_schema::model::{MigrationStep, RelationSpec};

impl EngineSession {
    /// Prepare one explicitly selected, registered schema transformation.
    /// The input role must bind its exact source version. Execution is explicit and
    /// retains ordinary residual relation obligations before publication.
    ///
    /// # Errors
    /// Unknown declaration, absent source, incompatible fields/defaults, native
    /// planning failure, cancellation or resource exhaustion.
    pub fn prepare_schema_transform(
        &self,
        declaration: &str,
        input_role: &str,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        cancel.checkpoint()?;
        let registry = self.registry();
        let declaration = registry
            .migrations()
            .iter()
            .find(|candidate| candidate.qualified_name() == declaration)
            .ok_or_else(|| invalid("schema transformation is not registered"))?;
        let endpoint = |version| {
            registry
                .relations()
                .iter()
                .find(|relation| {
                    relation.qualified_name() == declaration.relation
                        && relation.key.version == version
                })
                .ok_or_else(|| invalid("schema transformation endpoint is not declared"))
        };
        let source = endpoint(declaration.from_version)?;
        let target = endpoint(declaration.to_version)?;
        if !self
            .input_roles()
            .any(|(role, key)| role == input_role && key == source.key)
        {
            return Err(invalid(
                "input role does not bind the declared source version",
            ));
        }
        let input = self.scan_role(input_role)?;
        let input = output::forget_relation_annotations(input).map_err(engine)?;
        let mut columns = input
            .schema()
            .columns()
            .into_iter()
            .map(|column| (column.name().to_owned(), Expr::Column(column)))
            .collect::<Vec<_>>();
        for step in &declaration.steps {
            cancel.checkpoint()?;
            apply(step, target, registry, &mut columns)?;
        }
        let expressions = target
            .columns
            .iter()
            .map(|column| {
                columns
                    .iter()
                    .find(|(name, _)| name == column.name())
                    .map(|(_, expression)| expression.clone().alias(column.name()))
                    .ok_or_else(|| invalid("schema transformation output column is absent"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let plan = LogicalPlanBuilder::from(input)
            .project(expressions)
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        let plan = output::declare_relation_output(plan, registry, target).map_err(engine)?;
        self.prepare_rule_plan(plan, cancel)
    }
}

fn apply(
    step: &MigrationStep,
    target: &RelationSpec,
    registry: &pse_schema::Registry,
    columns: &mut Vec<(String, Expr)>,
) -> Result<(), EngineError> {
    match step {
        MigrationStep::AddColumn { name, default } => {
            let column = target
                .column(name)
                .ok_or_else(|| invalid("default column is undeclared"))?;
            let default = default
                .bind(registry)
                .map_err(pse_columnar::external)
                .map_err(engine)?;
            let value =
                output::checked_array_literal(registry, column, default.array()).map_err(engine)?;
            columns.push(((*name).to_owned(), value));
        }
        MigrationStep::DropColumn(name) => columns.retain(|(column, _)| column != name),
        MigrationStep::RenameColumn { from, to } => {
            (*to).clone_into(&mut column_mut(columns, from)?.0);
        }
        MigrationStep::ChangeNullable { name, nullable } => {
            let (_, value) = column_mut(columns, name)?;
            *value = if *nullable {
                scalar::nullable(value.clone())
            } else {
                scalar::require_nonnull(value.clone())
            };
        }
    }
    Ok(())
}

fn column_mut<'a>(
    columns: &'a mut [(String, Expr)],
    name: &str,
) -> Result<&'a mut (String, Expr), EngineError> {
    columns
        .iter_mut()
        .find(|(column, _)| column == name)
        .ok_or_else(|| invalid("schema transformation source column is absent"))
}

fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "schema transformation".to_owned(),
        reason: reason.to_owned(),
    }
}
