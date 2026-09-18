// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native occurrence queries over declared nested fields. Parent masks are applied
//! before extracting children, and absent/empty containers make no reference claims.

use datafusion::{
    arrow::datatypes::{DataType, Field},
    common::{Column, DataFusionError, NullHandling, Result, UnnestOptions},
    functions::core::expr_fn::{get_field, named_struct},
    functions_nested::expr_fn::{map_keys, map_values},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, cast, lit},
};
use std::collections::BTreeSet;

pub(super) struct Occurrence {
    pub field: Field,
    pub path: Vec<String>,
    pub input: LogicalPlan,
}
impl Occurrence {
    pub(super) fn value(&self) -> Result<Expr> {
        value(&self.input)
    }
}

/// The resulting plans expose one non-null column, preserving multiplicity.
/// Private names never shadow a source field or another occurrence projection.
pub(super) fn occurrences<'a>(
    input: &LogicalPlan,
    fields: impl IntoIterator<Item = &'a Field>,
    matches: fn(&Field) -> bool,
) -> Result<Vec<Occurrence>> {
    let mut output = vec![];
    let mut names = input
        .schema()
        .fields()
        .iter()
        .map(|field| field.name().clone())
        .collect();
    for field in fields {
        if contains(field, matches) {
            descend(
                field,
                vec![field.name().clone()],
                project(input.clone(), column(field.name()), &mut names)?,
                matches,
                &mut output,
                &mut names,
            )?;
        }
    }
    Ok(output)
}

fn contains(field: &Field, matches: fn(&Field) -> bool) -> bool {
    matches(field)
        || match field.data_type() {
            DataType::Struct(fields) => fields.iter().any(|child| contains(child, matches)),
            DataType::List(child)
            | DataType::LargeList(child)
            | DataType::ListView(child)
            | DataType::LargeListView(child)
            | DataType::FixedSizeList(child, _)
            | DataType::Map(child, _) => contains(child, matches),
            _ => false,
        }
}

fn descend(
    field: &Field,
    path: Vec<String>,
    input: LogicalPlan,
    matches: fn(&Field) -> bool,
    output: &mut Vec<Occurrence>,
    names: &mut BTreeSet<String>,
) -> Result<()> {
    let source = value(&input)?;
    let input = LogicalPlanBuilder::from(input)
        .filter(source.clone().is_not_null())?
        .build()?;
    if matches(field) {
        output.push(Occurrence {
            field: field.clone(),
            path: path.clone(),
            input: input.clone(),
        });
    }
    match field.data_type() {
        DataType::Struct(fields) => {
            for child in fields.iter().filter(|child| contains(child, matches)) {
                let mut child_path = path.clone();
                child_path.push(child.name().clone());
                descend(
                    child,
                    child_path,
                    project(
                        input.clone(),
                        get_field(source.clone(), child.name()),
                        names,
                    )?,
                    matches,
                    output,
                    names,
                )?;
            }
        }
        DataType::Map(child, _) if contains(child, matches) => {
            let input = map_input(input, child, names)?;
            let mut child_path = path;
            child_path.push("[]".into());
            descend(child, child_path, input, matches, output, names)?;
        }
        DataType::List(child)
        | DataType::LargeList(child)
        | DataType::ListView(child)
        | DataType::LargeListView(child)
        | DataType::FixedSizeList(child, _)
            if contains(child, matches) =>
        {
            let input = match field.data_type() {
                DataType::ListView(child) => {
                    project(input, cast(source, DataType::List(child.clone())), names)?
                }
                DataType::LargeListView(child) => project(
                    input,
                    cast(source, DataType::LargeList(child.clone())),
                    names,
                )?,
                _ => input,
            };
            let name = input.schema().field(0).name().clone();
            let input = LogicalPlanBuilder::from(input)
                .unnest_column_with_options(
                    name,
                    UnnestOptions::new().with_null_handling(NullHandling::Drop),
                )?
                .build()?;
            let mut child_path = path;
            child_path.push("[]".into());
            descend(child, child_path, input, matches, output, names)?;
        }
        _ => {}
    }
    Ok(())
}

fn map_input(
    input: LogicalPlan,
    entries: &Field,
    names: &mut BTreeSet<String>,
) -> Result<LogicalPlan> {
    let DataType::Struct(fields) = entries.data_type() else {
        return Err(DataFusionError::Plan("map entries must be a struct".into()));
    };
    let [key, value] = fields.as_ref() else {
        return Err(DataFusionError::Plan(
            "map entries require a key and value".into(),
        ));
    };
    // map_entries normalizes child names at this pin. Paired native UNNEST over
    // the same map preserves entry correlation and arbitrary declared names.
    let source = self::value(&input)?;
    let key_name = fresh_name(names);
    let value_name = fresh_name(names);
    let input = LogicalPlanBuilder::from(input)
        .project(vec![
            map_keys(source.clone()).alias(&key_name),
            map_values(source).alias(&value_name),
        ])?
        .unnest_columns_with_options(
            vec![Column::from_name(&key_name), Column::from_name(&value_name)],
            UnnestOptions::new().with_null_handling(NullHandling::Drop),
        )?
        .build()?;
    project(
        input,
        named_struct(vec![
            lit(key.name().clone()),
            column(&key_name),
            lit(value.name().clone()),
            column(&value_name),
        ]),
        names,
    )
}

fn project(input: LogicalPlan, value: Expr, names: &mut BTreeSet<String>) -> Result<LogicalPlan> {
    LogicalPlanBuilder::from(input)
        .project(vec![value.alias(fresh_name(names))])?
        .alias("__pse_nested_occurrence")?
        .build()
}
fn fresh_name(names: &mut BTreeSet<String>) -> String {
    let mut index = names.len();
    loop {
        let name = format!("__pse_nested_value_{index}");
        if names.insert(name.clone()) {
            return name;
        }
        index += 1;
    }
}
fn value(input: &LogicalPlan) -> Result<Expr> {
    let mut columns = input.schema().columns().into_iter();
    let column = columns
        .next()
        .ok_or_else(|| DataFusionError::Internal("nested occurrence column absent".into()))?;
    if columns.next().is_some() {
        return Err(DataFusionError::Internal(
            "nested occurrence requires one column".into(),
        ));
    }
    Ok(Expr::Column(column))
}
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}

#[cfg(test)]
mod tests;
