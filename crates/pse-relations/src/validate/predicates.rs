// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared local value domains lowered to ordinary DataFusion expressions.
//! Scalar predicates also become derived Delta SQL CHECK text. Collection expressions remain
//! native and use the narrowly scoped adapter required by Delta's pinned SQL parser.

use crate::native::{
    arrow::datatypes::{ArrowPrimitiveType, DataType, Field, Float16Type, Schema},
    common::{
        Column, DFSchema, DataFusionError, Result, ScalarValue,
        tree_node::{Transformed, TransformedResult, TreeNode, TreeNodeRecursion},
    },
    functions::{
        core::expr_fn::get_field, encoding::expr_fn::encode, math::expr_fn::gcd,
        string::expr_fn::octet_length,
    },
    functions_nested::expr_fn::{
        array_any_match, array_distinct, array_length, map_keys, map_values,
    },
    logical_expr::{
        BinaryExpr, Expr, Operator, binary_expr, cast,
        expr_fn::{lambda, lambda_var},
        lit,
        utils::{disjunction, split_binary_owned},
    },
};
use pse_schema::{
    Registry,
    model::{FieldContract, IntegerRange},
};

/// Explicit resolved enum domains, owned independently of the current registry.
#[derive(Clone, Debug, Default)]
pub struct DomainEnvironment {
    members: std::collections::BTreeMap<String, Vec<String>>,
}
impl DomainEnvironment {
    /// Resolve current declarations for the shared predicate compiler.
    pub fn current(registry: &Registry) -> Self {
        Self {
            members: registry
                .enums()
                .iter()
                .map(|domain| {
                    (
                        domain.name.to_owned(),
                        domain.members.iter().map(|m| m.name.to_owned()).collect(),
                    )
                })
                .collect(),
        }
    }
    /// Preserve the verified recorded domains instead of expanding them from today's registry.
    /// # Errors
    /// Inconsistent domain definitions across the support graph.
    pub fn recorded(
        recorded: &pse_schema::compatibility::VerifiedRecordedContract,
    ) -> Result<Self> {
        let mut members = std::collections::BTreeMap::new();
        for description in recorded.contract().relations.values() {
            let domains = description["enums"]
                .as_object()
                .ok_or_else(|| DataFusionError::Plan("missing recorded enum domains".into()))?;
            for (name, domain) in domains {
                let values: Vec<String> = domain["members"]
                    .as_object()
                    .ok_or_else(|| DataFusionError::Plan("invalid recorded enum members".into()))?
                    .keys()
                    .cloned()
                    .collect();
                if members.get(name).is_some_and(|old| old != &values) {
                    return Err(DataFusionError::Plan(format!(
                        "contradictory recorded enum {name}"
                    )));
                }
                members.insert(name.clone(), values);
            }
        }
        Ok(Self { members })
    }
}
/// Compile native local predicates for current declarations.
/// # Errors
/// Invalid domains or native expression binding failures.
pub fn relation(registry: &Registry, schema: &Schema) -> Result<Expr> {
    relation_in(&DomainEnvironment::current(registry), schema)
}
/// Compile all fields against an explicitly resolved semantic environment.
/// # Errors
/// Unknown enum domains or native expression binding failures.
pub fn relation_in(domains: &DomainEnvironment, schema: &Schema) -> Result<Expr> {
    let checks = schema
        .fields()
        .iter()
        .map(|field| field_value_in(domains, field, column(field.name()), 0))
        .collect::<Result<Vec<_>>>()?;
    Ok(combine(checks)
        .resolve_lambda_variables(&DFSchema::try_from(schema.clone())?)?
        .data)
}

/// Balance conjunction depth for a set of required native checks.
pub fn combine(checks: Vec<Expr>) -> Expr {
    all(checks)
}

/// Rebalance every AND/OR chain for SQL text. Field predicates use flat left-deep
/// chains, which native planning linearizes; the SQL unparser
/// parenthesizes each binary operator, so SQL text receives logarithmic nesting.
/// # Errors
/// Native expression traversal failure.
pub fn balanced(expr: Expr) -> Result<Expr> {
    expr.transform_down(|expr| {
        let Expr::BinaryExpr(BinaryExpr {
            op: op @ (Operator::And | Operator::Or),
            ..
        }) = expr
        else {
            return Ok(Transformed::no(expr));
        };
        let operands = split_binary_owned(expr, op)
            .into_iter()
            .map(balanced)
            .collect::<Result<Vec<_>>>()?;
        let tree = balance(operands, op)
            .ok_or_else(|| DataFusionError::Internal("a binary chain has operands".into()))?;
        Ok(Transformed::new(tree, true, TreeNodeRecursion::Jump))
    })
    .data()
}

fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}
fn all(checks: impl IntoIterator<Item = Expr>) -> Expr {
    balance(checks.into_iter().collect(), Operator::And).unwrap_or_else(|| lit(true))
}
fn balance(mut operands: Vec<Expr>, op: Operator) -> Option<Expr> {
    while operands.len() > 1 {
        let mut values = operands.into_iter();
        let mut next = Vec::new();
        while let Some(left) = values.next() {
            next.push(match values.next() {
                Some(right) => binary_expr(left, op, right),
                None => left,
            });
        }
        operands = next;
    }
    operands.pop()
}

/// Compile a visible field contract, including native nested children.
/// # Errors
/// Invalid domains or unsupported native expression construction.
pub fn field_value(registry: &Registry, field: &Field, value: Expr, depth: usize) -> Result<Expr> {
    field_value_in(&DomainEnvironment::current(registry), field, value, depth)
}
/// Compile nested field obligations from current or verified recorded domains.
/// # Errors
/// Unknown enum domains or unsupported native expression construction.
pub fn field_value_in(
    domains: &DomainEnvironment,
    field: &Field,
    value: Expr,
    depth: usize,
) -> Result<Expr> {
    field_predicate(domains, field, value, depth, true)
}
/// Compile only this field's own current obligations; traversal owns descendants.
/// # Errors
/// Unknown domains or unsupported native expressions.
pub fn field_local(registry: &Registry, field: &Field, value: Expr) -> Result<Expr> {
    field_local_in(&DomainEnvironment::current(registry), field, value)
}
/// Compile a local obligation from explicit resolved domains.
/// # Errors
/// Unknown domains or unsupported native expressions.
pub fn field_local_in(domains: &DomainEnvironment, field: &Field, value: Expr) -> Result<Expr> {
    field_predicate(domains, field, value, 0, false)
}

/// A field holds exactly when none of its violations does. Every violation is a
/// total Boolean, so the negation is never NULL.
fn field_predicate(
    domains: &DomainEnvironment,
    field: &Field,
    value: Expr,
    depth: usize,
    descendants: bool,
) -> Result<Expr> {
    let mut violations = vec![];
    field_violations(domains, field, value, depth, descendants, &mut violations)?;
    Ok(disjunction(violations).map_or_else(|| lit(true), |violated| !violated))
}

/// Append the total violation disjuncts of one field value.
///
/// Violations form one flat disjunction per nullable value or lambda body: a required
/// value contributes `value IS NULL` and splices its own and its descendants'
/// violations into the enclosing disjunction; a nullable value guards them with a
/// single CASE, which also keeps descendants of absent values unevaluated. The
/// expression depth therefore grows with nullable and collection nesting only, and
/// each flat disjunction remains one compiler obligation.
fn field_violations(
    domains: &DomainEnvironment,
    field: &Field,
    value: Expr,
    depth: usize,
    descendants: bool,
    out: &mut Vec<Expr>,
) -> Result<()> {
    if !field.is_nullable() {
        out.push(value.clone().is_null());
        return value_violations(domains, field, &value, depth, descendants, out);
    }
    let mut present = vec![];
    value_violations(domains, field, &value, depth, descendants, &mut present)?;
    if let Some(violated) = disjunction(present) {
        let present =
            crate::native::logical_expr::when(value.is_null(), lit(false)).otherwise(violated)?;
        out.push(present);
    }
    Ok(())
}

/// Violations of a present value. A local check violates unless it is TRUE.
fn value_violations(
    domains: &DomainEnvironment,
    field: &Field,
    value: &Expr,
    depth: usize,
    descendants: bool,
    out: &mut Vec<Expr>,
) -> Result<()> {
    if let Some(reference) =
        pse_schema::model::ReferenceContract::from_field(field).map_err(external)?
    {
        out.push(reference_presence(&reference, value).is_not_true());
    }
    if let Some(collection) =
        pse_schema::model::CollectionContract::from_field(field).map_err(external)?
    {
        out.extend(
            collection_checks(collection, value.clone())
                .into_iter()
                .map(Expr::is_not_true),
        );
    }
    if let Some(alternative) =
        pse_schema::model::TaggedAlternative::from_field(field).map_err(external)?
    {
        out.extend(
            alternative_checks(&alternative, value)?
                .into_iter()
                .map(Expr::is_not_true),
        );
    }
    if let Some(range) = IntegerRange::from_field(field).map_err(external)? {
        out.push(
            value
                .clone()
                .between(lit(range.minimum), lit(range.maximum))
                .is_not_true(),
        );
    }
    if let Some(name) = FieldContract::from_field(field.clone()).enum_name() {
        let members = domains
            .members
            .get(name)
            .ok_or_else(|| DataFusionError::Plan(format!("unknown enum {name}")))?;
        out.push(
            value
                .clone()
                .in_list(
                    members.iter().map(|member| lit(member.clone())).collect(),
                    false,
                )
                .is_not_true(),
        );
    }
    storage_violations(domains, field.data_type(), value, depth, descendants, out)?;
    match field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .map(String::as_str)
    {
        Some("pse.bound") => {
            let kind = get_field(value.clone(), "kind");
            let payload = get_field(value.clone(), "value");
            out.push(
                kind.clone()
                    .eq(lit("finite"))
                    .and(payload.clone().is_not_null())
                    .or(kind.eq(lit("unbounded")).and(payload.is_null()))
                    .is_not_true(),
            );
        }
        Some("pse.source_span") => {
            out.push(
                get_field(value.clone(), "start")
                    .lt_eq(get_field(value.clone(), "end"))
                    .is_not_true(),
            );
        }
        Some("pse.dimension_vector") => {
            let parameter = format!("pse_exponent_{depth}");
            let member = lambda_var(&parameter);
            let numerator = cast(get_field(member.clone(), "num"), DataType::Int64);
            let denominator = cast(get_field(member, "den"), DataType::Int64);
            let valid = denominator
                .clone()
                .gt(lit(0_i64))
                .and(gcd(numerator, denominator).eq(lit(1_i64)));
            out.push(array_any_match(
                value.clone(),
                lambda([parameter], valid.is_not_true()),
            ));
        }
        _ => {}
    }
    Ok(())
}

fn alternative_checks(
    alternative: &pse_schema::model::TaggedAlternative,
    value: &Expr,
) -> Result<Vec<Expr>> {
    let mut checks = vec![];
    let tag = get_field(value.clone(), &alternative.discriminator);
    checks.push(
        tag.clone()
            .in_list(alternative.arms.keys().cloned().map(lit).collect(), false),
    );
    // Each payload is present exactly when its tag selects it. This is linear
    // in the declared arms, including shared payloads and payload-free tags.
    for arm in alternative.payloads() {
        let selected = alternative
            .arms
            .iter()
            .filter(|(_, selected)| selected.as_deref() == Some(arm))
            .map(|(tag, _)| lit(tag.clone()))
            .collect();
        let payload = get_field(value.clone(), arm);
        checks.push(
            crate::native::logical_expr::when(
                tag.clone().in_list(selected, false),
                payload.clone().is_not_null(),
            )
            .otherwise(payload.is_null())?,
        );
    }
    Ok(checks)
}

fn collection_checks(collection: pse_schema::model::CollectionContract, value: Expr) -> Vec<Expr> {
    let length = array_length(value.clone());
    let mut checks = vec![length.clone().gt_eq(lit(collection.minimum))];
    if let Some(maximum) = collection.maximum {
        checks.push(length.clone().lt_eq(lit(maximum)));
    }
    if collection.unique {
        checks.push(length.eq(array_length(array_distinct(value))));
    }
    checks
}

/// A collection violates when any member violates: one higher-order call per level.
fn members_violation(
    domains: &DomainEnvironment,
    values: Expr,
    member: &Field,
    parameter: String,
    depth: usize,
) -> Result<Option<Expr>> {
    let mut violations = vec![];
    field_violations(
        domains,
        member,
        lambda_var(&parameter),
        depth,
        true,
        &mut violations,
    )?;
    Ok(disjunction(violations)
        .map(|violated| array_any_match(values, lambda([parameter], violated))))
}

fn storage_violations(
    domains: &DomainEnvironment,
    ty: &DataType,
    value: &Expr,
    depth: usize,
    descendants: bool,
    out: &mut Vec<Expr>,
) -> Result<()> {
    let mut check = |check: Expr| out.push(check.is_not_true());
    match ty {
        DataType::Struct(fields) if descendants => {
            for child in fields {
                field_violations(
                    domains,
                    child,
                    get_field(value.clone(), child.name()),
                    depth + 1,
                    true,
                    out,
                )?;
            }
        }
        DataType::List(child)
        | DataType::LargeList(child)
        | DataType::ListView(child)
        | DataType::LargeListView(child)
        | DataType::FixedSizeList(child, _)
            if descendants =>
        {
            if let DataType::FixedSizeList(_, width) = ty {
                check(array_length(value.clone()).eq(lit(i64::from(*width))));
            }
            out.extend(members_violation(
                domains,
                value.clone(),
                child,
                format!("pse_item_{depth}"),
                depth + 1,
            )?);
        }
        DataType::FixedSizeBinary(width) => {
            check(octet_length(encode(value.clone(), lit("hex"))).eq(lit(i64::from(*width) * 2)));
        }
        DataType::Map(entries, _) => {
            let DataType::Struct(fields) = entries.data_type() else {
                return Err(DataFusionError::Plan("map entries require a struct".into()));
            };
            let [key, item] = fields.as_ref() else {
                return Err(DataFusionError::Plan(
                    "map entries require key and value fields".into(),
                ));
            };
            let keys = map_keys(value.clone());
            check(array_length(array_distinct(keys.clone())).eq(array_length(keys.clone())));
            if descendants {
                for (values, child) in [(keys, key), (map_values(value.clone()), item)] {
                    out.extend(members_violation(
                        domains,
                        values,
                        child,
                        format!("pse_map_{depth}"),
                        depth + 1,
                    )?);
                }
            }
        }
        DataType::Float16 => {
            let maximum = f32::from(<Float16Type as ArrowPrimitiveType>::Native::MAX);
            check(cast(value.clone(), DataType::Float32).between(lit(-maximum), lit(maximum)));
        }
        DataType::Float64 => check(value.clone().between(lit(-f64::MAX), lit(f64::MAX))),
        DataType::Float32 => check(value.clone().between(lit(-f32::MAX), lit(f32::MAX))),
        DataType::UInt8 => check(value.clone().between(lit(0_i64), lit(i64::from(u8::MAX)))),
        DataType::UInt16 => check(value.clone().between(lit(0_i64), lit(i64::from(u16::MAX)))),
        DataType::UInt32 => check(value.clone().between(lit(0_i64), lit(i64::from(u32::MAX)))),
        DataType::UInt64 => check(value.clone().between(
            lit(0_i64),
            lit(ScalarValue::Decimal128(Some(i128::from(u64::MAX)), 20, 0)),
        )),
        DataType::Dictionary(_, values) => {
            storage_violations(domains, values, value, depth, descendants, out)?;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod consolidation_unit;

fn external(
    error: impl pse_diagnostics::TypedDiagnostic + Send + Sync + 'static,
) -> DataFusionError {
    pse_columnar::external(error)
}

/// Presence semantics shared by local validation and relational references.
pub fn reference_presence(reference: &pse_schema::model::ReferenceContract, value: &Expr) -> Expr {
    let components = reference
        .columns
        .iter()
        .map(|mapping| mapping.source.iter().fold(value.clone(), get_field))
        .collect::<Vec<_>>();
    let present = all(components.iter().cloned().map(Expr::is_not_null));
    match reference.null_policy {
        pse_schema::model::ReferenceNullPolicy::Required => present,
        pse_schema::model::ReferenceNullPolicy::AllOrNone => {
            present.or(all(components.into_iter().map(Expr::is_null)))
        }
    }
}
