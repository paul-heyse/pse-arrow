// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native parameter-coordinate joins over actual immutable method/domain inputs.
use crate::{
    CompilerError,
    passes::{
        native_construction::{append, c, error, filter, join, prefix, project},
        native_outputs::{SourceKey, Sources},
    },
};
use datafusion::{
    arrow::array::{Array, BooleanArray, FixedSizeBinaryArray, ListArray},
    functions::core::expr_fn::coalesce,
    functions_aggregate::expr_fn::count,
    functions_nested::expr_fn::array_length,
    logical_expr::{JoinType, LogicalPlan, LogicalPlanBuilder, col, lit, when},
};
use pse_catalog::session::scalar::array_element;
use pse_catalog::session::{SnapshotSession, scalar};
use pse_ids::{CancellationToken, MemoryReserver, ReservationLease, SemanticId};
use pse_schema::{Registry, model::RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct ProjectedIndex {
    pub(crate) values: Vec<SemanticId>,
    pub(crate) sources: BTreeSet<SourceKey>,
    _lease: Arc<ReservationLease>,
}
pub(crate) struct ParameterIndexProjector<'a> {
    registry: &'a Registry,
    session: SnapshotSession,
    sources: Sources,
    reserver: &'a dyn MemoryReserver,
}
impl<'a> ParameterIndexProjector<'a> {
    pub(crate) fn new(
        registry: &'a Registry,
        session: &SnapshotSession,
        sources: &Sources,
        reserver: &'a dyn MemoryReserver,
    ) -> Self {
        Self {
            registry,
            session: session.clone(),
            sources: sources.clone(),
            reserver,
        }
    }
    pub(crate) async fn project(
        &self,
        method: SemanticId,
        parameter: &str,
        domains: &[SemanticId],
        members: &[SemanticId],
        cancel: &CancellationToken,
    ) -> Result<ProjectedIndex, CompilerError> {
        cancel.checkpoint()?;
        if domains.len() != members.len() {
            return Err(invalid("parameter tuple and domain dimensions differ"));
        }
        let mut lease = self.reserver.open("method:native-parameter-coordinate");
        lease
            .try_grow(
                domains
                    .len()
                    .checked_mul(8192)
                    .and_then(|n| n.checked_add(8192))
                    .ok_or_else(|| invalid("parameter coordinate plan extent overflow"))?,
            )
            .map_err(pse_ids::CanonError::from)?;
        let mut roles = BTreeMap::new();
        let parameter_rows = self.scan("reference.method_parameters", "parameter", &mut roles)?;
        let parameter_rows = filter(
            parameter_rows,
            c("parameter", "method_id")
                .eq(self.sid(method)?)
                .and(c("parameter", "name").eq(lit(parameter))),
        )?;
        let mappings = self.scan(
            "reference.method_parameter_axes",
            "mapping",
            &mut BTreeMap::new(),
        )?;
        let mappings = filter(
            mappings,
            c("mapping", "method_id")
                .eq(self.sid(method)?)
                .and(c("mapping", "parameter_kind").eq(lit(parameter))),
        )?;
        let counts = LogicalPlanBuilder::from(mappings.clone())
            .aggregate(
                Vec::<datafusion::logical_expr::Expr>::new(),
                [count(lit(1_u64)).alias("mapping_count")],
            )
            .and_then(LogicalPlanBuilder::build)
            .map_err(error)?;
        let extent =
            u64::try_from(domains.len()).map_err(|_| invalid("parameter dimension overflow"))?;
        let joined = join(parameter_rows, counts, JoinType::Inner, [lit(true)])?;
        let mut plan = append(
            joined,
            [
                array_length(c("parameter", "indexed_by"))
                    .eq(lit(extent))
                    .and(col("mapping_count").eq(lit(extent)))
                    .alias("valid"),
                scalar::id_list(vec![]).alias("value"),
            ],
        )?;
        for (position, (domain, member)) in domains.iter().zip(members).enumerate() {
            let suffix = position.to_string();
            let mapping_alias = format!("mapping{suffix}");
            let domain_alias = format!("domain{suffix}");
            let member_alias = format!("member{suffix}");
            let material_alias = format!("material{suffix}");
            let mapping = self.scan(
                "reference.method_parameter_axes",
                &mapping_alias,
                &mut roles,
            )?;
            plan = join(
                plan,
                mapping,
                JoinType::Left,
                [
                    c(&mapping_alias, "method_id").eq(self.sid(method)?),
                    c(&mapping_alias, "parameter_kind").eq(lit(parameter)),
                    c(&mapping_alias, "position").eq(lit(u64::try_from(position)
                        .map_err(|_| invalid("parameter dimension overflow"))?)),
                ],
            )?;
            plan = join(
                plan,
                self.scan("normalized.domains", &domain_alias, &mut roles)?,
                JoinType::Left,
                [c(&domain_alias, "domain_id").eq(self.sid(*domain)?)],
            )?;
            plan = join(
                plan,
                self.scan("normalized.domain_members", &member_alias, &mut roles)?,
                JoinType::Left,
                [
                    c(&member_alias, "domain_id").eq(self.sid(*domain)?),
                    c(&member_alias, "member_id").eq(self.sid(*member)?),
                ],
            )?;
            let coordinate = c(&mapping_alias, "source_coordinate");
            let pair = coordinate.clone().eq(lit("phase_species_pair"));
            plan = join(
                plan,
                self.scan(
                    "normalized.material_domain_members",
                    &material_alias,
                    &mut roles,
                )?,
                JoinType::Left,
                [
                    pair.clone(),
                    c(&material_alias, "domain_id").eq(self.sid(*domain)?),
                    c(&material_alias, "member_id").eq(self.sid(*member)?),
                ],
            )?;
            let valid = col("valid")
                .and(c(&member_alias, "member_id").is_not_null())
                .and(
                    c(&domain_alias, "kind").eq(array_element(
                        c("parameter", "indexed_by"),
                        lit(i64::try_from(position + 1)
                            .map_err(|_| invalid("parameter dimension overflow"))?),
                    )),
                )
                .and(
                    coordinate
                        .clone()
                        .eq(lit("member"))
                        .or(coordinate
                            .clone()
                            .eq(lit("ref_entity"))
                            .and(c(&member_alias, "ref_entity_id").is_not_null()))
                        .or(pair
                            .clone()
                            .and(c(&domain_alias, "kind").eq(lit("phase_species")))
                            .and(c(&material_alias, "phase_id").is_not_null())
                            .and(c(&material_alias, "species_id").is_not_null())),
                );
            let value = when(
                coordinate.clone().eq(lit("member")),
                scalar::id_list(vec![c(&member_alias, "member_id")]),
            )
            .when(
                coordinate.eq(lit("ref_entity")),
                scalar::id_list(vec![c(&member_alias, "ref_entity_id")]),
            )
            .otherwise(scalar::id_list(vec![
                c(&material_alias, "phase_id"),
                c(&material_alias, "species_id"),
            ]))
            .map_err(error)?;
            let value = self
                .session
                .scalar_function("pse_array_concat")?
                .call(vec![col("value"), value]);
            let fields = plan
                .schema()
                .fields()
                .iter()
                .map(|field| field.name().to_owned())
                .collect::<Vec<_>>();
            plan = project(
                plan,
                fields.into_iter().map(|name| match name.as_str() {
                    "valid" => valid.clone().alias("valid"),
                    "value" => value.clone().alias("value"),
                    _ => col(name),
                }),
            )?;
        }
        let fields = [
            coalesce(vec![col("valid"), lit(false)]).alias("valid"),
            col("value"),
        ]
        .into_iter()
        .chain(roles.keys().map(|alias| c(alias, "source_key")));
        let complete = self
            .session
            .prepare_rule_plan(project(plan, fields)?, cancel)?
            .execute(cancel)
            .await?;
        let count = complete
            .batches()
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>();
        if count != 1 {
            return Err(invalid(
                "parameter coordinate has missing or ambiguous actual declarations",
            ));
        }
        let mut values = Vec::new();
        let mut sources = Vec::new();
        for batch in complete
            .batches()
            .iter()
            .filter(|batch| batch.num_rows() != 0)
        {
            lease
                .try_grow(pse_ids::validation_extent(batch)?)
                .map_err(pse_ids::CanonError::from)?;
            let valid = batch
                .column(0)
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| invalid("parameter validity storage differs"))?;
            if valid.is_null(0) || !valid.value(0) {
                return Err(invalid(
                    "parameter axes require complete ordered mappings, matching kinds and actual member/reference coordinates",
                ));
            }
            let list = batch
                .column(1)
                .as_any()
                .downcast_ref::<ListArray>()
                .ok_or_else(|| invalid("parameter coordinate storage differs"))?;
            if list.is_null(0) {
                return Err(invalid("parameter coordinate is null"));
            }
            let ids = list.value(0);
            let ids = ids
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .ok_or_else(|| invalid("parameter coordinate identity storage differs"))?;
            for row in 0..ids.len() {
                if ids.is_null(row) {
                    return Err(invalid("parameter coordinate member is null"));
                }
                values.push(
                    SemanticId::try_from_slice(ids.value(row))
                        .map_err(|_| invalid("parameter coordinate identity width differs"))?,
                );
            }
            for (position, (alias, relation)) in roles.iter().enumerate() {
                let keys = batch
                    .column(position + 2)
                    .as_any()
                    .downcast_ref::<FixedSizeBinaryArray>()
                    .ok_or_else(|| invalid("parameter source key storage differs"))?;
                // A material row is used only for phase/species coordinates. The
                // source key was nullable at the native left-join boundary.
                if keys.is_null(0) {
                    if alias.starts_with("material") {
                        continue;
                    }
                    return Err(invalid("parameter source key is absent"));
                }
                sources.push((
                    *relation,
                    crate::passes::native_rows::key_value(keys.value(0))?,
                ));
            }
        }
        Ok(ProjectedIndex {
            values,
            sources: self.sources.locate(sources)?,
            _lease: ReservationLease::new(lease),
        })
    }
    fn scan(
        &self,
        name: &str,
        alias: &str,
        roles: &mut BTreeMap<String, RelationKey>,
    ) -> Result<LogicalPlan, CompilerError> {
        let spec = self
            .registry
            .relation(name)
            .ok_or_else(|| invalid("parameter source declaration absent"))?;
        if self.sources.get(&spec.key).is_none() {
            return Err(invalid("parameter source has no actual owner"));
        }
        roles.insert(alias.to_owned(), spec.key);
        let plan = LogicalPlanBuilder::scan(
            self.session.table_reference(&spec.key)?,
            self.session.table_source(&spec.key)?,
            None,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
        prefix(
            append(
                plan,
                [scalar::key(
                    spec.id,
                    spec.primary_key
                        .iter()
                        .map(|name| (*name, col(*name)))
                        .collect(),
                )
                .alias("source_key")],
            )?,
            alias,
        )
    }
    fn sid(&self, value: SemanticId) -> Result<datafusion::logical_expr::Expr, CompilerError> {
        pse_catalog::session::output::checked_literal(
            self.registry,
            &pse_schema::model::FieldContract::payload(
                "id",
                pse_schema::model::FieldContract::id(),
                "Actual parameter coordinate identity.",
            ),
            datafusion::common::ScalarValue::FixedSizeBinary(16, Some(value.as_bytes().to_vec())),
        )
        .map_err(error)
    }
}
fn invalid(reason: impl Into<String>) -> CompilerError {
    pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.into(),
    }
    .into()
}
