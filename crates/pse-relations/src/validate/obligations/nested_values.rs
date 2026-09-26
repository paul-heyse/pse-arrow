// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native occurrence plans retain source columns, multiplicity and actual nested indices.
use crate::native::{
    arrow::datatypes::{DataType, Field},
    common::{Column, DataFusionError, NullHandling, Result, UnnestOptions},
    functions::{
        core::expr_fn::{get_field, named_struct, union_extract},
        string::expr_fn::concat,
    },
    functions_nested::expr_fn::{array_length, map_keys, map_values, range},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, cast, lit},
};
use std::collections::BTreeSet;

pub(super) struct Occurrence {
    pub path: Vec<String>,
    pub path_column: String,
    pub input: LogicalPlan,
}
impl Occurrence {
    pub(super) fn value(&self) -> Result<Expr> {
        value(&self.input)
    }
}
/// Private output names cannot shadow source fields. Column zero is the selected value.
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
            let path_column = fresh_name(&mut names);
            let mut roots = vec![column(field.name()).alias(fresh_name(&mut names))];
            roots.extend(
                input
                    .schema()
                    .columns()
                    .into_iter()
                    .filter(|column| column.name() != field.name())
                    .map(|column| Expr::Column(column).alias(fresh_name(&mut names))),
            );
            let input = LogicalPlanBuilder::from(input.clone())
                .project(roots)?
                .build()?;
            let input = append(
                input,
                vec![lit(format!("/{}", pointer(field.name()))).alias(&path_column)],
            )?;
            descend(
                field,
                vec![field.name().clone()],
                path_column,
                input,
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
        || pse_schema::model::field::child_fields(field.data_type())
            .into_iter()
            .any(|child| contains(child, matches))
}
#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive native container lowering"
)]
fn descend(
    field: &Field,
    path: Vec<String>,
    path_column: String,
    input: LogicalPlan,
    matches: fn(&Field) -> bool,
    output: &mut Vec<Occurrence>,
    names: &mut BTreeSet<String>,
) -> Result<()> {
    let source = value(&input)?;
    let input = LogicalPlanBuilder::from(input)
        .filter(source.clone().is_not_null())?
        .build()?;
    if matches(field) && !matches!(field.data_type(), DataType::Dictionary(..)) {
        output.push(Occurrence {
            path: path.clone(),
            path_column: path_column.clone(),
            input: input.clone(),
        });
    }
    match field.data_type() {
        DataType::Struct(fields) => {
            let alternative = pse_schema::model::TaggedAlternative::from_field(field)
                .map_err(pse_columnar::external)?;
            for child in fields.iter().filter(|child| contains(child, matches)) {
                let mut selected = input.clone();
                if let Some(alternative) = &alternative
                    && alternative.payloads().contains(child.name().as_str())
                {
                    let tags = alternative
                        .arms
                        .iter()
                        .filter(|(_, arm)| arm.as_deref() == Some(child.name()))
                        .map(|(tag, _)| lit(tag.clone()))
                        .collect();
                    selected = LogicalPlanBuilder::from(selected)
                        .filter(
                            get_field(source.clone(), &alternative.discriminator)
                                .in_list(tags, false),
                        )?
                        .build()?;
                }
                let child_path_name = fresh_name(names);
                let selected = append(
                    selected,
                    vec![
                        concat(vec![
                            column(&path_column),
                            lit(format!("/{}", pointer(child.name()))),
                        ])
                        .alias(&child_path_name),
                    ],
                )?;
                let mut child_path = path.clone();
                child_path.push(child.name().clone());
                descend(
                    child,
                    child_path,
                    child_path_name,
                    project(selected, get_field(source.clone(), child.name()), names)?,
                    matches,
                    output,
                    names,
                )?;
            }
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
            let ordinal = fresh_name(names);
            let input = append(
                input,
                vec![
                    range(
                        lit(0_i64),
                        cast(array_length(column(&name)), DataType::Int64),
                        lit(1_i64),
                    )
                    .alias(&ordinal),
                ],
            )?;
            let input = LogicalPlanBuilder::from(input)
                .unnest_columns_with_options(
                    vec![Column::from_name(name), Column::from_name(&ordinal)],
                    UnnestOptions::new().with_null_handling(NullHandling::Drop),
                )?
                .build()?;
            let child_path_name = fresh_name(names);
            let input = append(
                input,
                vec![
                    concat(vec![
                        column(&path_column),
                        lit("/"),
                        cast(column(&ordinal), DataType::Utf8),
                    ])
                    .alias(&child_path_name),
                ],
            )?;
            let mut child_path = path;
            child_path.push("[]".into());
            descend(
                child,
                child_path,
                child_path_name,
                input,
                matches,
                output,
                names,
            )?;
        }
        DataType::Map(child, _) if contains(child, matches) => {
            let DataType::Struct(fields) = child.data_type() else {
                return Err(invalid("map entries must be a struct"));
            };
            let [key, item] = fields.as_ref() else {
                return Err(invalid("map entries require two fields"));
            };
            let key_name = fresh_name(names);
            let item_name = fresh_name(names);
            let ordinal = fresh_name(names);
            let keys = map_keys(source.clone());
            let input = append(
                input,
                vec![
                    keys.clone().alias(&key_name),
                    map_values(source).alias(&item_name),
                    range(
                        lit(0_i64),
                        cast(array_length(keys), DataType::Int64),
                        lit(1_i64),
                    )
                    .alias(&ordinal),
                ],
            )?;
            let input = LogicalPlanBuilder::from(input)
                .unnest_columns_with_options(
                    vec![
                        Column::from_name(&key_name),
                        Column::from_name(&item_name),
                        Column::from_name(&ordinal),
                    ],
                    UnnestOptions::new().with_null_handling(NullHandling::Drop),
                )?
                .build()?;
            let child_path_name = fresh_name(names);
            let input = append(
                input,
                vec![
                    concat(vec![
                        column(&path_column),
                        lit("/"),
                        cast(column(&ordinal), DataType::Utf8),
                    ])
                    .alias(&child_path_name),
                ],
            )?;
            let input = project(
                input,
                named_struct(vec![
                    lit(key.name().clone()),
                    column(&key_name),
                    lit(item.name().clone()),
                    column(&item_name),
                ]),
                names,
            )?;
            let mut child_path = path;
            child_path.push("[]".into());
            descend(
                child,
                child_path,
                child_path_name,
                input,
                matches,
                output,
                names,
            )?;
        }
        DataType::Union(fields, _) => {
            for (_, child) in fields.iter().filter(|(_, child)| contains(child, matches)) {
                let child_path_name = fresh_name(names);
                let selected = append(
                    input.clone(),
                    vec![
                        concat(vec![
                            column(&path_column),
                            lit(format!("/{}", pointer(child.name()))),
                        ])
                        .alias(&child_path_name),
                    ],
                )?;
                let mut child_path = path.clone();
                child_path.push(child.name().clone());
                descend(
                    child,
                    child_path,
                    child_path_name,
                    project(
                        selected,
                        union_extract(source.clone(), child.name().clone()),
                        names,
                    )?,
                    matches,
                    output,
                    names,
                )?;
            }
        }
        DataType::RunEndEncoded(_, child) if contains(child, matches) => {
            descend(
                child,
                path,
                path_column,
                project(input, cast(source, child.data_type().clone()), names)?,
                matches,
                output,
                names,
            )?;
        }
        DataType::Dictionary(_, kind) => {
            let decoded = field.clone().with_data_type(kind.as_ref().clone());
            descend(
                &decoded,
                path,
                path_column,
                project(input, cast(source, kind.as_ref().clone()), names)?,
                matches,
                output,
                names,
            )?;
        }
        _ => {}
    }
    Ok(())
}
fn project(input: LogicalPlan, value: Expr, names: &mut BTreeSet<String>) -> Result<LogicalPlan> {
    let mut columns = vec![value.alias(fresh_name(names))];
    columns.extend(input.schema().columns().into_iter().map(Expr::Column));
    LogicalPlanBuilder::from(input)
        .project(columns)?
        .alias("__pse_nested_occurrence")?
        .build()
}
pub(super) fn append(input: LogicalPlan, extra: Vec<Expr>) -> Result<LogicalPlan> {
    let mut columns = input
        .schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect::<Vec<_>>();
    columns.extend(extra);
    LogicalPlanBuilder::from(input).project(columns)?.build()
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
    input
        .schema()
        .columns()
        .into_iter()
        .next()
        .map(Expr::Column)
        .ok_or_else(|| invalid("native occurrence column absent"))
}
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}
fn pointer(name: &str) -> String {
    name.replace('~', "~0").replace('/', "~1")
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
#[cfg(test)]
mod consolidation_unit;
