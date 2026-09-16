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

pub(super) struct Occurrence {
    pub field: Field,
    pub path: Vec<String>,
    pub input: LogicalPlan,
}

/// The resulting plans expose one non-null `value` column, preserving multiplicity.
pub(super) fn occurrences<'a>(
    input: &LogicalPlan,
    fields: impl IntoIterator<Item = &'a Field>,
    matches: fn(&Field) -> bool,
) -> Result<Vec<Occurrence>> {
    let mut output = vec![];
    for field in fields {
        if contains(field, matches) {
            descend(
                field,
                vec![field.name().clone()],
                project(input.clone(), column(field.name()))?,
                matches,
                &mut output,
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
) -> Result<()> {
    let input = LogicalPlanBuilder::from(input)
        .filter(column("value").is_not_null())?
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
                    project(input.clone(), get_field(column("value"), child.name()))?,
                    matches,
                    output,
                )?;
            }
        }
        DataType::Map(child, _) if contains(child, matches) => {
            let input = map_input(input, child)?;
            let mut child_path = path;
            child_path.push("[]".into());
            descend(child, child_path, input, matches, output)?;
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
                    project(input, cast(column("value"), DataType::List(child.clone())))?
                }
                DataType::LargeListView(child) => project(
                    input,
                    cast(column("value"), DataType::LargeList(child.clone())),
                )?,
                _ => input,
            };
            let input = LogicalPlanBuilder::from(input)
                .unnest_column_with_options(
                    "value",
                    UnnestOptions::new().with_null_handling(NullHandling::Drop),
                )?
                .build()?;
            let mut child_path = path;
            child_path.push("[]".into());
            descend(child, child_path, input, matches, output)?;
        }
        _ => {}
    }
    Ok(())
}

fn map_input(input: LogicalPlan, entries: &Field) -> Result<LogicalPlan> {
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
    let input = LogicalPlanBuilder::from(input)
        .project(vec![
            map_keys(column("value")).alias("map_key"),
            map_values(column("value")).alias("map_value"),
        ])?
        .unnest_columns_with_options(
            vec![Column::from_name("map_key"), Column::from_name("map_value")],
            UnnestOptions::new().with_null_handling(NullHandling::Drop),
        )?
        .build()?;
    project(
        input,
        named_struct(vec![
            lit(key.name().clone()),
            column("map_key"),
            lit(value.name().clone()),
            column("map_value"),
        ]),
    )
}

fn project(input: LogicalPlan, value: Expr) -> Result<LogicalPlan> {
    LogicalPlanBuilder::from(input)
        .project(vec![value.alias("value")])?
        .build()
}
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}

#[cfg(test)]
mod tests;
