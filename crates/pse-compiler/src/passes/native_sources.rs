// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bind native source columns to their exact immutable input or completed producer.

use crate::{AlgorithmInputs, CompilerError};
use datafusion::functions::core::expr_fn::coalesce;
use datafusion::{
    common::{Column, ScalarValue},
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{SnapshotSession, output::checked_literal};
use pse_ids::CancellationToken;
use pse_rules::strata::{LocatedRuleInput, RuleInputLocation, native_input::NativeWitness};
use pse_schema::{
    Registry,
    model::{AlgorithmSpec, FieldContract},
};
use std::{collections::BTreeMap, ops::Not};

pub(crate) fn name(alias: &str, field: &str) -> String {
    format!("{}:{alias}:{field}", alias.len())
}

pub(crate) struct Sources<'a> {
    pub(crate) session: &'a SnapshotSession,
    pub(crate) inputs: &'a AlgorithmInputs,
    pub(crate) pass: &'a AlgorithmSpec,
    witnesses: BTreeMap<String, NativeWitness>,
}
impl<'a> Sources<'a> {
    pub(crate) fn new(
        session: &'a SnapshotSession,
        inputs: &'a AlgorithmInputs,
        pass: &'a AlgorithmSpec,
    ) -> Self {
        Self {
            session,
            inputs,
            pass,
            witnesses: BTreeMap::new(),
        }
    }
    pub(crate) fn scan(
        &mut self,
        relation: &str,
        alias: &str,
    ) -> Result<LogicalPlan, CompilerError> {
        let spec = self
            .session
            .registry()
            .relation(relation)
            .ok_or_else(|| invalid(format!("native source relation {relation} absent")))?;
        let input_port = self
            .pass
            .inputs
            .iter()
            .find(|port| port.relation == relation);
        let (port, location) = {
            let port = input_port
                .ok_or_else(|| invalid(format!("native plan reads undeclared input {relation}")))?;
            let bound = self
                .inputs
                .port(&port.port)
                .and_then(Option::as_ref)
                .ok_or_else(|| invalid(format!("native source input {} absent", port.port)))?;
            let location = RuleInputLocation::Facts(std::sync::Arc::clone(bound.relation()?));
            (port.port.as_str(), location)
        };
        let input = LocatedRuleInput {
            relation: spec.key,
            location,
        };
        let keys = spec
            .primary_key
            .iter()
            .map(|key| name(alias, key))
            .collect::<Vec<_>>();
        let mut presence = name(alias, "__native_row_present");
        while spec
            .columns
            .iter()
            .any(|field| name(alias, field.name()) == presence)
        {
            presence.push('_');
        }
        let present = keys
            .iter()
            .map(|key| Expr::Column(Column::from_name(key)).is_not_null())
            .reduce(Expr::and)
            .unwrap_or_else(|| col(&presence).is_not_null());
        if self
            .witnesses
            .insert(
                format!("row:{alias}"),
                NativeWitness {
                    port: port.to_owned(),
                    input: input.clone(),
                    key_columns: Some(keys),
                    when: Some(present),
                },
            )
            .is_some()
        {
            return Err(invalid(format!("native source alias {alias} is reused")));
        }
        self.witnesses
            .entry(format!("scope:{port}"))
            .or_insert_with(|| NativeWitness {
                port: port.to_owned(),
                input,
                key_columns: None,
                when: None,
            });
        let source = LogicalPlanBuilder::scan(
            self.session.table_reference(&spec.key)?,
            self.session.table_source(&spec.key)?,
            None,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(super::native_rows::engine)?;
        let mut columns = spec
            .columns
            .iter()
            .map(|field| col(field.name()).alias(name(alias, field.name())))
            .collect::<Vec<_>>();
        if spec.primary_key.is_empty() {
            columns.push(lit(true).alias(presence));
        }
        LogicalPlanBuilder::from(source)
            .project(columns)
            .and_then(LogicalPlanBuilder::build)
            .map_err(super::native_rows::engine)
    }
    pub(crate) fn witnesses(&self, plan: &LogicalPlan) -> Vec<NativeWitness> {
        self.witnesses
            .values()
            .filter(|witness| {
                witness
                    .key_columns
                    .iter()
                    .flatten()
                    .all(|name| plan.schema().field_with_unqualified_name(name).is_ok())
                    && witness.when.as_ref().is_none_or(|predicate| {
                        predicate
                            .column_refs()
                            .iter()
                            .all(|column| plan.schema().qualified_field_from_column(column).is_ok())
                    })
            })
            .cloned()
            .collect()
    }
}

pub(crate) fn error(error: datafusion::common::DataFusionError) -> CompilerError {
    pse_rules::errmap::classify(error, pse_catalog::PlanOrigin::RuleCompiler).into()
}
pub(crate) fn invalid(message: impl Into<String>) -> CompilerError {
    super::p4::invalid(message)
}
pub(crate) fn c(alias: &str, field: &str) -> Expr {
    Expr::Column(Column::from_name(name(alias, field)))
}
pub(crate) fn sid(registry: &Registry, id: pse_ids::SemanticId) -> Result<Expr, CompilerError> {
    checked_literal(
        registry,
        &FieldContract::payload("id", FieldContract::id(), "Actual source identity."),
        ScalarValue::FixedSizeBinary(16, Some(id.as_bytes().to_vec())),
    )
    .map_err(error)
}
pub(crate) fn project(
    input: LogicalPlan,
    values: impl IntoIterator<Item = Expr>,
) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(input)
        .project(values)
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)
}
pub(crate) fn append(
    input: LogicalPlan,
    values: impl IntoIterator<Item = Expr>,
) -> Result<LogicalPlan, CompilerError> {
    let mut fields = input
        .schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect::<Vec<_>>();
    fields.extend(values);
    project(input, fields)
}
pub(crate) fn filter(input: LogicalPlan, condition: Expr) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(input)
        .filter(condition)
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)
}
pub(crate) fn join(
    left: LogicalPlan,
    right: LogicalPlan,
    kind: JoinType,
    on: impl IntoIterator<Item = Expr>,
) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(left)
        .join_on(right, kind, on)
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)
}
pub(crate) async fn require(
    input: &LogicalPlan,
    condition: Expr,
    message: &str,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<(), CompilerError> {
    let failures = filter(input.clone(), coalesce(vec![condition, lit(false)]).not())?;
    let result = session
        .prepare_rule_plan(failures, cancel)?
        .execute(cancel)
        .await?;
    if result.batches().iter().any(|batch| batch.num_rows() != 0) {
        return Err(invalid(message));
    }
    Ok(())
}
