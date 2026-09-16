// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native finite-product framing and exact source-key association.
mod axes;
use crate::{CompilerError, passes::native_outputs::Sources};
use datafusion::{
    common::{Column, ScalarValue, UnnestOptions},
    functions::core::expr_fn::coalesce,
    functions_nested::expr_fn::{array_length, make_array, range},
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, Operator, col, lit},
};
use pse_catalog::session::scalar::array_element;
use pse_catalog::session::{SnapshotSession, output::checked_literal, scalar};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::columnar::FieldCheckedBatch;
use pse_rules::strata::native_input::{NativeInput, NativeWitness};
use pse_schema::model::{FieldContract, PassSpec, RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Not,
    sync::Arc,
};

pub(crate) fn error(error: datafusion::common::DataFusionError) -> CompilerError {
    pse_rules::errmap::classify(error, pse_catalog::PlanOrigin::RuleCompiler).into()
}
pub(crate) fn invalid(reason: impl AsRef<str>) -> CompilerError {
    pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.as_ref().to_owned(),
    }
    .into()
}
pub(crate) fn c(alias: &str, field: &str) -> Expr {
    Expr::Column(Column::from_name(format!("{alias}:{field}")))
}
pub(crate) fn concat(left: Expr, right: Expr) -> Expr {
    Expr::BinaryExpr(datafusion::logical_expr::expr::BinaryExpr::new(
        Box::new(left),
        Operator::StringConcat,
        Box::new(right),
    ))
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
pub(crate) fn distinct(input: LogicalPlan) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(input)
        .distinct()
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)
}
pub(crate) fn union(inputs: Vec<LogicalPlan>) -> Result<LogicalPlan, CompilerError> {
    let mut inputs = inputs.into_iter();
    let mut result = inputs
        .next()
        .ok_or_else(|| invalid("product union has no declared branches"))?;
    for input in inputs {
        result = LogicalPlanBuilder::from(result)
            .union(input)
            .and_then(LogicalPlanBuilder::build)
            .map_err(error)?;
    }
    Ok(result)
}
pub(crate) fn prefix(input: LogicalPlan, alias: &str) -> Result<LogicalPlan, CompilerError> {
    let fields = input
        .schema()
        .iter()
        .map(|(qualifier, field)| {
            Expr::Column(Column::new(qualifier.cloned(), field.name()))
                .alias(format!("{alias}:{}", field.name()))
        })
        .collect::<Vec<_>>();
    project(input, fields)
}
pub(crate) fn explode(
    input: LogicalPlan,
    list: Expr,
    position: &str,
) -> Result<LogicalPlan, CompilerError> {
    let input = append(
        input,
        [range(lit(0_i64), array_length(list), lit(1_i64)).alias(position)],
    )?;
    LogicalPlanBuilder::from(input)
        .unnest_column_with_options(position, UnnestOptions::new().with_preserve_nulls(false))
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)
}

pub(crate) struct Plans<'a> {
    pub session: SnapshotSession,
    pub sources: Sources,
    pub pass: &'a PassSpec,
    pub cancel: &'a CancellationToken,
    pub used: BTreeSet<RelationKey>,
    ordinal: usize,
}
impl<'a> Plans<'a> {
    pub(crate) fn new(
        inputs: &BTreeMap<RelationKey, FieldCheckedBatch>,
        sources: &Sources,
        pass: &'a PassSpec,
        session: &SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> Result<Self, CompilerError> {
        let mut selected = BTreeMap::new();
        for (key, batch) in inputs {
            if !sources.contains_key(key) {
                return Err(invalid(format!(
                    "native source {} lacks its actual producer binding",
                    key.qualified_name()
                )));
            }
            selected.insert(*key, batch.clone());
        }
        let session = session
            .select_inputs(&BTreeSet::new(), cancel)?
            .with_checked_workspace(selected, cancel)?;
        Ok(Self {
            session,
            sources: sources.clone(),
            pass,
            cancel,
            used: BTreeSet::new(),
            ordinal: 0,
        })
    }
    pub(crate) fn key(&self, name: &str) -> Result<RelationKey, CompilerError> {
        Ok(self
            .session
            .registry()
            .relation(name)
            .ok_or_else(|| invalid("product relation undeclared"))?
            .key)
    }
    pub(crate) fn ids(&self, values: Vec<Expr>) -> Result<Expr, CompilerError> {
        Ok(self
            .session
            .scalar_function("pse_array_concat")?
            .call(values))
    }
    pub(crate) fn index(&self, values: Expr) -> Result<Expr, CompilerError> {
        Ok(self
            .session
            .scalar_function("pse_index_tuple")?
            .call(vec![values]))
    }
    pub(crate) fn lists(&self, values: Vec<Expr>) -> Result<Expr, CompilerError> {
        Ok(self
            .session
            .scalar_function("pse_array_concat")?
            .call(values))
    }
    pub(crate) fn present(&self, value: Expr) -> Result<Expr, CompilerError> {
        Ok(self
            .session
            .scalar_function("pse_require_nonnull")?
            .call(vec![value]))
    }
    pub(crate) fn sid(&self, id: SemanticId) -> Result<Expr, CompilerError> {
        self.literal(
            &FieldContract::payload("id", FieldContract::id(), "Actual source identity."),
            ScalarValue::FixedSizeBinary(16, Some(id.as_bytes().to_vec())),
        )
    }
    pub(crate) fn literal(
        &self,
        column: &FieldContract,
        value: ScalarValue,
    ) -> Result<Expr, CompilerError> {
        checked_literal(self.session.registry(), column, value).map_err(error)
    }
    pub(crate) fn enum_literal(
        &self,
        enumeration: &'static str,
        value: &str,
    ) -> Result<Expr, CompilerError> {
        let ty = FieldContract::enumeration(enumeration);
        let scalar = ScalarValue::Utf8(Some(value.to_owned()))
            .cast_to(&ty.data_type())
            .map_err(error)?;
        self.literal(
            &FieldContract::payload("enum_value", ty, "Declared enum member"),
            scalar,
        )
    }
    pub(crate) fn scan(&mut self, name: &str, alias: &str) -> Result<LogicalPlan, CompilerError> {
        let spec = self
            .session
            .registry()
            .relation(name)
            .ok_or_else(|| invalid(format!("product source {name} undeclared")))?;
        self.used.insert(spec.key);
        let input = LogicalPlanBuilder::scan(
            self.session.table_reference(&spec.key)?,
            self.session.table_source(&spec.key)?,
            None,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
        let token = concat(
            lit(format!("{}:{name}:", name.len())),
            scalar::key(
                spec.primary_key
                    .iter()
                    .map(|name| (*name, col(*name)))
                    .collect(),
            ),
        );
        prefix(
            append(
                input,
                [
                    token.clone().alias("source_token"),
                    make_array(vec![token]).alias("support"),
                ],
            )?,
            alias,
        )
    }
    pub(crate) async fn require(
        &self,
        input: &LogicalPlan,
        condition: Expr,
        message: &str,
    ) -> Result<(), CompilerError> {
        let failed = filter(input.clone(), coalesce(vec![condition, lit(false)]).not())?;
        let complete = self
            .session
            .prepare_rule_plan(failed, self.cancel)
            .map_err(|source| pse_rules::RuleError::Execution {
                rule: format!("{}: {message}", self.pass.name),
                phase: "native obligation preparation",
                source: Box::new(source.into()),
            })?
            .execute(self.cancel)
            .await?;
        if complete.batches().iter().any(|batch| batch.num_rows() != 0) {
            return Err(invalid(message));
        }
        Ok(())
    }
    pub(crate) async fn retain(
        &mut self,
        input: LogicalPlan,
    ) -> Result<LogicalPlan, CompilerError> {
        Ok(self.retain_with_count(input).await?.0)
    }
    /// Retain one actual execution and its observed cardinality, including empty
    /// frontiers. The count is read from that completion, never a replay.
    pub(crate) async fn retain_with_count(
        &mut self,
        input: LogicalPlan,
    ) -> Result<(LogicalPlan, usize), CompilerError> {
        let complete = self
            .session
            .prepare_rule_plan(input, self.cancel)
            .map_err(|source| pse_rules::RuleError::Execution {
                rule: format!("{}: frontier {}", self.pass.name, self.ordinal),
                phase: "native frontier preparation",
                source: Box::new(source.into()),
            })?
            .execute(self.cancel)
            .await?;
        let count = complete.batches().iter().try_fold(0usize, |count, batch| {
            count
                .checked_add(batch.num_rows())
                .ok_or_else(|| invalid("native frontier cardinality overflow"))
        })?;
        let role = format!("native-construction-framing-{}", self.ordinal);
        self.ordinal = self
            .ordinal
            .checked_add(1)
            .ok_or_else(|| invalid("native construction ordinal overflow"))?;
        self.session = self
            .session
            .with_computation_roles(BTreeMap::from([(role.clone(), complete)]), self.cancel)?;
        Ok((self.session.scan_computation_role(&role)?, count))
    }
    fn source_columns(
        &self,
        mut plan: LogicalPlan,
        keys: Vec<Expr>,
        ids: Vec<(Expr, Expr)>,
    ) -> Result<LogicalPlan, CompilerError> {
        let mut field =
            FieldContract::payload("id", FieldContract::id(), "Unmatched source identity.");
        field = field.optional();
        let mut source_id = checked_literal(
            self.session.registry(),
            &field,
            ScalarValue::FixedSizeBinary(16, None),
        )
        .map_err(error)?;
        for (condition, id) in ids.into_iter().rev() {
            source_id = pse_catalog::session::output::same_field_case(
                plan.schema(),
                condition,
                id,
                source_id,
            )
            .map_err(error)?;
        }
        plan = append(
            plan,
            [
                self.present(source_id)?.alias("source_relation_id"),
                self.present(coalesce(keys))?.alias("source_key"),
            ],
        )?;
        Ok(plan)
    }

    async fn source_support(
        &mut self,
        plan: LogicalPlan,
        source_columns: bool,
    ) -> Result<(LogicalPlan, Vec<NativeWitness>), CompilerError> {
        let plan = explode(plan, col("supports"), "support_position")?;
        let mut plan = append(
            plan,
            [
                array_element(col("supports"), col("support_position") + lit(1_i64))
                    .alias("support_token"),
            ],
        )?;
        let mut witnesses = Vec::new();
        let mut keys = Vec::new();
        let mut ids = Vec::new();
        let mut found = Vec::new();
        for (index, key) in self.used.clone().iter().enumerate() {
            let (port, input) = self
                .sources
                .get(key)
                .ok_or_else(|| invalid("product source has no retained location"))?
                .clone();
            let relation = self
                .session
                .registry()
                .relation(&key.qualified_name())
                .ok_or_else(|| invalid("product source declaration absent"))?
                .clone();
            let alias = format!("witness_{index}");
            let source = self.scan(&key.qualified_name(), &alias)?;
            plan = join(
                plan,
                source,
                JoinType::Left,
                [col("support_token").eq(c(&alias, "source_token"))],
            )?;
            let when = c(&alias, "source_token").is_not_null();
            found.push(when.clone());
            keys.push(
                datafusion::logical_expr::when(
                    when.clone(),
                    scalar::key(
                        relation
                            .primary_key
                            .iter()
                            .map(|name| (*name, c(&alias, name)))
                            .collect(),
                    ),
                )
                .otherwise(lit(ScalarValue::Utf8(None)))
                .map_err(error)?,
            );
            ids.push((when.clone(), self.sid(relation.id)?));
            witnesses.push(NativeWitness {
                port: port.clone(),
                input: input.clone(),
                key_columns: relation
                    .primary_key
                    .iter()
                    .map(|name| format!("{alias}:{name}"))
                    .collect(),
                when: Some(when),
            });
            witnesses.push(NativeWitness {
                port,
                input,
                key_columns: vec![],
                when: None,
            });
        }
        let known = found
            .into_iter()
            .reduce(Expr::or)
            .unwrap_or_else(|| lit(false));
        self.require(
            &plan,
            known,
            "product construction has an unbound exact source key",
        )
        .await?;
        if source_columns {
            plan = self.source_columns(plan, keys, ids)?;
        }
        Ok((plan, witnesses))
    }

    /// Expand exact keys and join them to immutable source owners before minting a
    /// native output. No key string or identifier alone certifies membership.
    pub(crate) async fn output(
        &mut self,
        name: &str,
        plan: LogicalPlan,
        source_columns: bool,
    ) -> Result<Arc<NativeInput>, CompilerError> {
        let target = self
            .session
            .registry()
            .relation(name)
            .ok_or_else(|| invalid("product output undeclared"))?
            .clone();
        let (mut plan, witnesses) = self.source_support(plan, source_columns).await?;
        if target.column("derivation_id").is_some() {
            plan = append(plan, [self.sid(SemanticId::NIL)?.alias("derivation_id")])?;
        }
        let values = target
            .columns
            .iter()
            .map(|field| {
                let value = if field.nullable() {
                    col(field.name())
                } else {
                    self.present(col(field.name()))?
                };
                Ok(value.alias(format!("result_{}", field.name())))
            })
            .collect::<Result<Vec<_>, CompilerError>>()?;
        plan = append(plan, values)?;
        let columns = target
            .columns
            .iter()
            .map(|field| (field.name().to_owned(), format!("result_{}", field.name())))
            .collect();
        let output = NativeInput::build(
            plan,
            target.key,
            self.pass.id,
            columns,
            witnesses,
            &self.session,
            self.cancel,
        )
        .await?;
        let selected = self
            .session
            .input_keys()
            .filter(|key| *key != target.key)
            .collect();
        self.session = self
            .session
            .select_inputs(&selected, self.cancel)?
            .with_checked_workspace(
                BTreeMap::from([(target.key, output.checked().clone())]),
                self.cancel,
            )?;
        self.sources
            .replace_native(target.key, Arc::clone(&output))?;
        Ok(output)
    }
}
