// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The bounded algebra maps to actual engine operators, not row-wise emulation.
use super::{Column, Compiler, Planned, expr};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion_common::{
    Column as EngineColumn, JoinType, NullEquality as EngineNullEquality, UnnestOptions,
};
use datafusion_expr::{ExprSchemable, LogicalPlanBuilder, col};
use pse_schema::model::{FieldContract, NullEquality, NullListPolicy, RulePlan};

impl Compiler<'_> {
    pub(super) fn lower(&mut self, source: &RulePlan) -> Result<Planned, RuleError> {
        if let Some((_, output)) = self
            .native
            .results
            .iter()
            .rev()
            .find(|(plan, _)| super::identity::same_plan(plan, source))
        {
            return Ok(output.clone());
        }
        match source {
            RulePlan::Assert { .. } => Err(internal(
                "assertion predicate must be split into native truth queries before lowering",
            )),
            RulePlan::Scan { relation, port } => self.scan(relation, port),
            RulePlan::RecursiveRef { name } => self
                .binders
                .iter()
                .rev()
                .find(|(candidate, _)| candidate == name)
                .map(|(_, planned)| planned.clone())
                .ok_or_else(|| internal("recursive reference is outside its lexical step")),
            RulePlan::Filter { input, predicate } => {
                let mut input = self.lower(input)?;
                let (predicate, ty) = expr::lower(predicate, &input, self.registry, self.session)?;
                expr::boolean(&ty)?;
                input.plan = LogicalPlanBuilder::from(input.plan)
                    .filter(predicate)
                    .map_err(engine)?
                    .build()
                    .map_err(engine)?;
                input.plan = pse_catalog::session::scalar::refine_filtered_fields(input.plan)
                    .map_err(engine)?;
                for column in &mut input.columns {
                    column.spec = column.spec.clone().with_nullable(
                        expr::column_expression(column)
                            .nullable(input.plan.schema())
                            .map_err(engine)?,
                    );
                }
                Ok(input)
            }
            RulePlan::Project { input, columns } => self.project(input, columns),

            RulePlan::EquiJoin {
                left,
                right,
                keys,
                null_equality,
            } => self.join(left, right, keys, false, *null_equality),
            RulePlan::AntiJoin { left, right, keys } => {
                self.join(left, right, keys, true, NullEquality::NullEqualsNothing)
            }
            RulePlan::Distinct(input) => {
                let mut input = self.lower(input)?;
                for column in &input.columns {
                    self.key(column)?;
                }
                input.plan = LogicalPlanBuilder::from(input.plan)
                    .distinct()
                    .map_err(engine)?
                    .build()
                    .map_err(engine)?;
                Ok(input)
            }
            RulePlan::Union(inputs) => self.union(inputs),
            RulePlan::Aggregate {
                input,
                group,
                aggregates,
            } => self.aggregate(input, group, aggregates),
            RulePlan::Unnest {
                input,
                column,
                value_name,
                null_list,
                ..
            } => self.unnest(input, column, value_name.clone(), *null_list),
            RulePlan::Recursive {
                name,
                seed,
                step,
                is_distinct,
                depth_bound,
            } => self.recursive(name, seed, step, *is_distinct, *depth_bound),
        }
    }
    fn project(
        &mut self,
        input: &RulePlan,
        columns: &[(std::borrow::Cow<'static, str>, pse_schema::model::RuleExpr)],
    ) -> Result<Planned, RuleError> {
        let input = self.lower(input)?;
        let mut expressions = vec![];
        let mut contracts = vec![];
        for (name, expression) in columns {
            let (value, mut contract) =
                expr::lower(expression, &input, self.registry, self.session)?;
            // A direct same-name column already carries its field meaning.
            // Retain its native qualification instead of wrapping it in an
            // identity metadata alias. At this pin leaf-expression pushdown
            // treats Alias(Column) as a computation and may append the same
            // source column again beside it while merging projections.
            let passthrough = matches!(&value, datafusion_expr::Expr::Column(column)
                        if column.name == name.as_ref());
            contract.name.clone_from(name);
            if passthrough {
                expressions.push(value);
            } else {
                let field = pse_schema::arrow::field_for(self.registry, &contract.spec)
                    .map_err(|error| internal(error.to_string()))?;
                expressions.push(value.alias_with_metadata(
                    name.as_ref(),
                    Some(datafusion_common::metadata::FieldMetadata::from(&field)),
                ));
                contract.qualifier = None;
                contract.physical = None;
            }
            contracts.push(contract);
        }
        expressions.extend(input.hidden.iter().map(col));
        let plan = LogicalPlanBuilder::from(input.plan)
            .project(expressions)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        Ok(Planned {
            plan,
            columns: contracts,
            hidden: input.hidden,
        })
    }
    fn union(&mut self, inputs: &[RulePlan]) -> Result<Planned, RuleError> {
        let mut inputs = inputs
            .iter()
            .map(|input| self.lower(input))
            .collect::<Result<Vec<_>, _>>()?;
        let first = inputs.first().ok_or_else(|| internal("empty union"))?;
        let mut columns = first.columns.clone();
        for input in &inputs[1..] {
            if columns.len() != input.columns.len() || first.hidden != input.hidden {
                return Err(internal("union branches have different column inventories"));
            }
            for (left, right) in columns.iter_mut().zip(&input.columns) {
                if left.name != right.name
                    || left.spec.value_type() != right.spec.value_type()
                    || left.spec.quantity() != right.spec.quantity()
                {
                    return Err(internal("union branches have different semantic types"));
                }
                left.spec = left
                    .spec
                    .clone()
                    .with_nullable(left.spec.nullable() || right.spec.nullable());
                if !super::identity::same_literal(left.literal.as_ref(), right.literal.as_ref()) {
                    left.literal = None;
                }
                // A union cannot inherit one branch's source key role or foreign
                // key when another branch carries no such contract. Preserve only
                // relational claims proved by every input; semantic types above
                // remain exact, including nested metadata and quantity contracts.
                if left.spec.role() != right.spec.role() {
                    left.spec = left
                        .spec
                        .clone()
                        .with_role(pse_schema::model::ColumnRole::Payload);
                }
                if left.spec.fk() != right.spec.fk() {
                    left.spec = left.spec.clone().without_fk();
                }
            }
        }
        for input in &mut inputs {
            let mut expressions = input
                .columns
                .iter()
                .zip(&columns)
                .map(|(source, target)| {
                    let field = pse_schema::arrow::field_for(self.registry, &target.spec)
                        .map_err(|error| internal(error.to_string()))?;
                    Ok(expr::column_expression(source).alias_with_metadata(
                        target.name.as_ref(),
                        Some(datafusion_common::metadata::FieldMetadata::from(&field)),
                    ))
                })
                .collect::<Result<Vec<_>, RuleError>>()?;
            expressions.extend(input.hidden.iter().map(col));
            input.plan = LogicalPlanBuilder::from(input.plan.clone())
                .project(expressions)
                .map_err(engine)?
                .build()
                .map_err(engine)?;
        }
        for column in &mut columns {
            column.qualifier = None;
            column.physical = None;
        }
        let mut inputs = inputs.into_iter();
        let mut output = inputs.next().ok_or_else(|| internal("empty union"))?;
        output.columns = columns;
        for input in inputs {
            output.plan = LogicalPlanBuilder::from(output.plan)
                .union(input.plan)
                .map_err(engine)?
                .build()
                .map_err(engine)?;
        }
        Ok(output)
    }
    fn scan(&self, relation: &str, port: &str) -> Result<Planned, RuleError> {
        let key = self
            .binding
            .ports
            .get(port)
            .ok_or_else(|| internal(format!("unbound rule port {port}")))?;
        if key.qualified_name() != relation {
            return Err(internal(format!(
                "port {port} is bound to {key}, not {relation}"
            )));
        }
        let spec = self
            .registry
            .relation(relation)
            .ok_or_else(|| internal("scan relation is not declared"))?;
        if &spec.key != key {
            return Err(internal(
                "scan version differs from the exact registry declaration",
            ));
        }
        let source = self.session.table_source(key)?;
        let reference = self.session.table_reference(key)?;
        let plan = LogicalPlanBuilder::scan(reference, source, None)
            .map_err(engine)?
            .alias(port)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let columns = spec
            .columns
            .iter()
            .map(|spec| Column {
                name: spec.name().to_owned().into(),
                spec: spec.clone(),
                qualifier: Some(port.to_owned()),
                physical: None,
                literal: None,
            })
            .collect();
        Ok(Planned {
            plan,
            columns,
            hidden: vec![],
        })
    }
    pub(super) fn key(&self, column: &Column) -> Result<(), RuleError> {
        if column.spec.value_type()
            == FieldContract::native(datafusion::arrow::datatypes::DataType::Float64)
        {
            return Err(RuleError::FloatKey {
                rule: self.rule.qualified_name(),
                column: column.name.to_string(),
            });
        }
        if !column.spec.value_type().admits_exact_key() {
            return Err(internal("rule key has no admitted exact equality"));
        }
        Ok(())
    }
    fn join(
        &mut self,
        left: &RulePlan,
        right: &RulePlan,
        keys: &[(
            std::borrow::Cow<'static, str>,
            std::borrow::Cow<'static, str>,
        )],
        anti: bool,
        null: NullEquality,
    ) -> Result<Planned, RuleError> {
        let mut left = self.lower(left)?;
        let mut right = self.lower(right)?;
        if keys.is_empty() {
            return Err(internal("rule joins require explicit equality keys"));
        }
        for (l, r) in keys {
            let left_key = expr::lookup(&left, l)?;
            let right_key = expr::lookup(&right, r)?;
            self.key(left_key)?;
            self.key(right_key)?;
            if left_key.spec.value_type() != right_key.spec.value_type()
                || left_key.spec.quantity() != right_key.spec.quantity()
            {
                return Err(internal("join key physical contracts differ"));
            }
        }
        self.scope_unqualified(&mut left)?;
        self.scope_unqualified(&mut right)?;
        let key = |input: &Planned, name: &str| {
            let column = expr::lookup(input, name)?;
            Ok(column.physical.clone().unwrap_or_else(|| {
                EngineColumn::new(column.qualifier.clone(), column.name.as_ref())
            }))
        };
        let join_keys = (
            keys.iter()
                .map(|(name, _)| key(&left, name))
                .collect::<Result<Vec<_>, RuleError>>()?,
            keys.iter()
                .map(|(_, name)| key(&right, name))
                .collect::<Result<Vec<_>, RuleError>>()?,
        );
        left.plan = LogicalPlanBuilder::from(left.plan)
            .join_detailed(
                right.plan,
                if anti {
                    JoinType::LeftAnti
                } else {
                    JoinType::Inner
                },
                join_keys,
                None,
                match null {
                    NullEquality::NullEqualsNothing => EngineNullEquality::NullEqualsNothing,
                    NullEquality::NullEqualsNull => EngineNullEquality::NullEqualsNull,
                },
            )
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        if !anti {
            left.columns.extend(right.columns);
            left.hidden.extend(right.hidden);
        }
        Ok(left)
    }
    /// DataFusion rejects an unqualified field beside a qualified field of the
    /// same name. Scope only those native addresses; rule lookup still resolves
    /// the declared projected name independently from each scan's port name.
    fn scope_unqualified(&mut self, input: &mut Planned) -> Result<(), RuleError> {
        if input.columns.iter().all(|column| {
            column
                .physical
                .as_ref()
                .map_or(column.qualifier.is_some(), |address| {
                    address.relation.is_some()
                })
        }) {
            return Ok(());
        }
        let qualifier = format!("__pse_binding_{}", self.binding_counter);
        self.binding_counter += 1;
        let mut projection = Vec::with_capacity(input.columns.len() + input.hidden.len());
        for column in &mut input.columns {
            let value = expr::column_expression(column);
            let datafusion_expr::Expr::Column(address) = &value else {
                return Err(internal("rule column binding is not a native column"));
            };
            if address.relation.is_none() {
                let physical_name = address.name.clone();
                projection.push(datafusion_expr::Expr::Alias(
                    datafusion_expr::expr::Alias::new(
                        value,
                        Some(qualifier.clone()),
                        &physical_name,
                    ),
                ));
                column.physical = Some(EngineColumn::new(Some(qualifier.clone()), physical_name));
            } else {
                projection.push(value);
            }
        }
        projection.extend(input.hidden.iter().map(col));
        input.plan = LogicalPlanBuilder::from(input.plan.clone())
            .project(projection)
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        Ok(())
    }
    fn unnest(
        &mut self,
        source: &RulePlan,
        name: &str,
        value_name: std::borrow::Cow<'static, str>,
        null: NullListPolicy,
    ) -> Result<Planned, RuleError> {
        let mut input = self.lower(source)?;
        let column = expr::lookup(&input, name)?;
        let (datafusion::arrow::datatypes::DataType::List(element)
        | datafusion::arrow::datatypes::DataType::FixedSizeList(element, _)) =
            column.spec.data_type()
        else {
            return Err(internal("unnest requires a declared list"));
        };
        let element = FieldContract::from_field((*element).clone());
        let value = expr::column_expression(column);
        if null == NullListPolicy::Reject && column.spec.nullable() {
            let check = LogicalPlanBuilder::from(input.plan.clone())
                .filter(value.clone().is_null())
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            self.checks
                .push((check, "unnest rejects null lists".to_owned()));
        }
        let mut projected: Vec<_> = input.columns.iter().map(expr::column_expression).collect();
        projected.extend(input.hidden.iter().map(col));
        projected.push(value.alias(value_name.as_ref()));
        input.plan = LogicalPlanBuilder::from(input.plan)
            .project(projected)
            .map_err(engine)?
            .unnest_columns_with_options(
                vec![EngineColumn::from_name(value_name.as_ref())],
                UnnestOptions::new().with_preserve_nulls(false),
            )
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        input.columns.push(Column {
            name: value_name,
            spec: FieldContract::payload("_member", element, "Declared list member"),
            qualifier: None,
            physical: None,
            literal: None,
        });
        // The common native preparation derives the element field from this
        // UNNEST's exact source child. A second self-alias projection neither adds
        // meaning nor changes values, and triggers the pinned leaf-pushdown alias
        // collision when a consumer extracts a member from a struct element.
        Ok(input)
    }
}
pub(super) fn compatible(left: &Planned, right: &Planned) -> Result<(), RuleError> {
    if left.columns.len() != right.columns.len()
        || left.columns.iter().zip(&right.columns).any(|(l, r)| {
            l.name != r.name
                || l.spec.value_type() != r.spec.value_type()
                || l.spec.quantity() != r.spec.quantity()
                || l.spec.nullable() != r.spec.nullable()
        })
        || left.hidden != right.hidden
    {
        return Err(internal(
            "union or recursive branches have different full column contracts",
        ));
    }
    Ok(())
}
