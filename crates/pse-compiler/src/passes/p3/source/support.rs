// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native-selected keys for the exact declarations read by source algorithms.
use super::{Origins, invalid};
use crate::{
    CompilerError,
    passes::{
        native_outputs::{GeneratedOutputs, OutputRows, SourceKey},
        native_rows::{AlgorithmInputs, Keyed, engine, keyed_rows, scan},
    },
};
use datafusion::{
    common::ScalarValue,
    logical_expr::{Expr, LogicalPlanBuilder, col, lit},
};
use pse_authoring::document::binding::{OwnedSourceBindings, PathMeaning, SourceExpression};
use pse_catalog::session::{SnapshotSession, output::checked_literal, scalar};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::{authored, normalized},
};
use pse_schema::model::RelationSpec;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Support<'a> {
    construction: &'a super::provenance::Construction,
    session: &'a SnapshotSession,
    cancel: &'a CancellationToken,
    arguments: AlgorithmInputs,
}
impl<'a> Support<'a> {
    pub(super) fn new(
        construction: &'a super::provenance::Construction,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> Self {
        Self {
            construction,
            session,
            cancel,
            arguments: AlgorithmInputs::new(session.reserver(), "P3:source-declarations"),
        }
    }
    fn key(
        &self,
        spec: &RelationSpec,
        key: pse_ids::ContentHash,
    ) -> Result<SourceKey, CompilerError> {
        let (port, _) = self
            .construction
            .sources
            .get(&spec.key)
            .ok_or_else(|| invalid("source declaration has no unique immutable role"))?;
        Ok(SourceKey {
            relation: spec.key,
            port: port.clone(),
            key,
        })
    }
    fn identity(
        &self,
        spec: &RelationSpec,
        field: &str,
        value: SemanticId,
    ) -> Result<Expr, CompilerError> {
        checked_literal(
            self.session.registry(),
            spec.column(field)
                .ok_or_else(|| invalid("source identity field absent"))?,
            ScalarValue::FixedSizeBinary(16, Some(value.as_bytes().to_vec())),
        )
        .map_err(engine)
    }
    async fn select<T: RelationRow>(
        &mut self,
        predicates: Vec<Expr>,
    ) -> Result<Vec<Keyed<T>>, CompilerError> {
        let spec = T::relation(self.session.registry())?;
        let mut plan = LogicalPlanBuilder::from(scan(self.session, spec, "source")?);
        for predicate in predicates {
            plan = plan.filter(predicate).map_err(engine)?;
        }
        keyed_rows(
            &mut self.arguments,
            plan.build().map_err(engine)?,
            self.session,
            self.session.registry(),
            self.cancel,
        )
        .await
    }
    async fn selected_keys(
        &mut self,
        name: &str,
        predicates: Vec<Expr>,
    ) -> Result<Origins, CompilerError> {
        let spec = self
            .session
            .registry()
            .relation(name)
            .ok_or_else(|| invalid("source declaration absent"))?;
        let mut plan = LogicalPlanBuilder::from(scan(self.session, spec, "source")?);
        for predicate in predicates {
            plan = plan.filter(predicate).map_err(engine)?;
        }
        let plan = plan
            .project([scalar::key(
                spec.id,
                spec.primary_key
                    .iter()
                    .map(|name| (*name, col(*name)))
                    .collect(),
            )
            .alias("source_key")])
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        self.arguments
            .keys(plan, self.session, self.cancel)
            .await?
            .into_iter()
            .map(|key| {
                self.key(
                    spec,
                    key.ok_or_else(|| invalid("source projection produced a null key"))?,
                )
            })
            .collect()
    }
    pub(super) async fn id(
        &mut self,
        name: &str,
        field: &str,
        id: SemanticId,
    ) -> Result<Origins, CompilerError> {
        let spec = self
            .session
            .registry()
            .relation(name)
            .ok_or_else(|| invalid("source declaration absent"))?;
        let origins = self
            .selected_keys(name, vec![col(field).eq(self.identity(spec, field, id)?)])
            .await?;
        if origins.len() != 1 {
            return Err(invalid(&format!(
                "{name}.{field} has no unique actual source"
            )));
        }
        Ok(origins)
    }
    async fn named(
        &mut self,
        relation: &str,
        template: SemanticId,
        name: &str,
    ) -> Result<Origins, CompilerError> {
        let spec = self
            .session
            .registry()
            .relation(relation)
            .ok_or_else(|| invalid("named source declaration absent"))?;
        let origins = self
            .selected_keys(
                relation,
                vec![
                    col("template_id").eq(self.identity(spec, "template_id", template)?),
                    col("name").eq(lit(name)),
                ],
            )
            .await?;
        if origins.len() != 1 {
            return Err(invalid("named source has no unique actual declaration"));
        }
        Ok(origins)
    }
    pub(super) async fn owner(
        &mut self,
        id: SemanticId,
    ) -> Result<(authored::templates::Row, SourceKey), CompilerError> {
        let spec = authored::templates::spec(self.session.registry())?;
        let mut rows = self
            .select::<authored::templates::Row>(vec![col("template_id").eq(self.identity(
                spec,
                "template_id",
                id,
            )?)])
            .await?;
        if rows.len() != 1 {
            return Err(invalid("expression owner template is absent or ambiguous"));
        }
        let row = rows
            .pop()
            .ok_or_else(|| invalid("source owner disappeared"))?;
        Ok((row.row, self.key(spec, row.key)?))
    }
    pub(super) async fn expression(
        &mut self,
        source: &SourceExpression,
        bindings: &OwnedSourceBindings,
    ) -> Result<Origins, CompilerError> {
        let spec = self
            .session
            .registry()
            .relation_by_id(source.relation_id)
            .ok_or_else(|| invalid("expression source declaration absent"))?;
        let key = bindings
            .source_key(source)
            .ok_or_else(|| invalid("expression has no native source correspondence"))?;
        let mut origins = Origins::from([self.key(spec, key.to_owned())?]);
        for path in &source.paths {
            let selected = match &path.meaning {
                PathMeaning::Symbol { symbol_id, .. } => {
                    self.id("authored.template_symbols", "symbol_decl_id", *symbol_id)
                        .await?
                }
                PathMeaning::Equation { equation_id, .. } => {
                    self.id(
                        "authored.template_equations",
                        "equation_decl_id",
                        *equation_id,
                    )
                    .await?
                }
                PathMeaning::Port { template_id, name } => {
                    self.named("authored.template_ports", *template_id, name)
                        .await?
                }
                PathMeaning::Domain { template_id, name } => {
                    self.named("authored.template_domains", *template_id, name)
                        .await?
                }
                PathMeaning::Parameter { template_id, name } => {
                    self.named("authored.template_params", *template_id, name)
                        .await?
                }
                PathMeaning::Feature { template_id, name } => {
                    self.named("authored.template_features", *template_id, name)
                        .await?
                }
                PathMeaning::Entity(id) => self.id("authored.entities", "entity_id", *id).await?,
                PathMeaning::Unit { unit_id } => {
                    self.id("reference.units", "unit_id", *unit_id).await?
                }
                PathMeaning::InstancePath(path) => {
                    let mut members = Origins::new();
                    for id in &path.member_entities {
                        members.extend(
                            self.id("authored.template_symbols", "symbol_decl_id", *id)
                                .await?,
                        );
                    }
                    members
                }
                PathMeaning::EnumLiteral { .. }
                | PathMeaning::BooleanLiteral(_)
                | PathMeaning::Local(_) => Origins::new(),
            };
            origins.extend(selected);
        }
        Ok(origins)
    }
    pub(super) async fn unit(
        &mut self,
        id: pse_quantity::UnitId,
        package: SemanticId,
        units: &super::units::Units<'_>,
    ) -> Result<Origins, CompilerError> {
        let mut origins = Origins::new();
        let mut pending = vec![id];
        let mut seen = BTreeSet::new();
        while let Some(id) = pending.pop() {
            self.cancel.checkpoint()?;
            if !seen.insert(id) {
                continue;
            }
            if let Some(dependencies) = units.dependencies.get(&id) {
                pending.extend(dependencies);
                let derived = units.derived.get(&id).ok_or_else(|| {
                    invalid("derived unit dependency lacks its actual definition")
                })?;
                origins.extend(
                    self.id("authored.package_unit_sets", "package_id", package)
                        .await?,
                );
                origins.extend(
                    self.id("reference.unit_sets", "unit_set_id", derived.unit_set_id)
                        .await?,
                );
            } else {
                origins.extend(self.id("reference.units", "unit_id", id.as_id()).await?);
            }
        }
        Ok(origins)
    }
    pub(super) async fn unit_set(
        &mut self,
        package: SemanticId,
        set: pse_quantity::UnitSetId,
    ) -> Result<Origins, CompilerError> {
        let mut origins = self
            .id("authored.package_unit_sets", "package_id", package)
            .await?;
        origins.extend(
            self.id("reference.unit_sets", "unit_set_id", set.as_id())
                .await?,
        );
        Ok(origins)
    }
    pub(super) async fn packages(
        &mut self,
        output: &FieldCheckedBatch,
    ) -> Result<GeneratedOutputs, CompilerError> {
        let spec = authored::packages::spec(self.session.registry())?;
        let headers = self.select::<authored::packages::Row>(Vec::new()).await?;
        let headers = headers
            .into_iter()
            .map(|header| {
                Ok((
                    header.row.package_id,
                    (header.row, self.key(spec, header.key)?),
                ))
            })
            .collect::<Result<BTreeMap<_, _>, CompilerError>>()?;
        let mut result = OutputRows::new(
            self.session.registry(),
            self.session.reserver(),
            self.cancel,
        )?;
        result.ensure::<normalized::package_graph::Row>()?;
        for row in normalized::package_graph::Row::rows(output)? {
            let mut seen = BTreeSet::new();
            let mut pending = vec![row.package_id];
            let mut origins = Origins::new();
            while let Some(id) = pending.pop() {
                self.cancel.checkpoint()?;
                if !seen.insert(id) {
                    continue;
                }
                let (header, key) = headers
                    .get(&id)
                    .ok_or_else(|| invalid("package depth depends on an absent header"))?;
                origins.insert(key.clone());
                pending.extend(
                    header
                        .dependencies
                        .iter()
                        .map(|dependency| dependency.package_id),
                );
            }
            result.push(row, &origins)?;
        }
        result.finish()
    }
}
