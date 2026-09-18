// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed law arguments and native keys from the same retained completion.
use super::expansion::invalid;
use crate::{
    AlgorithmContext, CompilerError,
    passes::{
        native_outputs::{SourceKey, Sources},
        native_rows::{AlgorithmInputs, keyed_rows, scan},
    },
};
use pse_catalog::session::SnapshotSession;
use pse_relations::{
    columnar::RelationRow,
    generated::{compiled, inferred, normalized, reference},
};

pub(super) use crate::passes::native_rows::Located;
pub(super) struct Inventory {
    pub session: SnapshotSession,
    pub sources: Sources,
    pub law_applications: Vec<Located<compiled::law_applications::Row>>,
    pub contributions: Vec<Located<compiled::contributions::Row>>,
    pub expression_root_indices: Vec<Located<compiled::expression_root_indices::Row>>,
    pub symbol_groups: Vec<Located<compiled::symbol_groups::Row>>,
    pub symbol_group_members: Vec<Located<compiled::symbol_group_members::Row>>,
    pub symbols: Vec<Located<compiled::symbols::Row>>,
    pub law_contexts: Vec<Located<inferred::law_contexts::Row>>,
    pub law_axes: Vec<Located<inferred::law_axes::Row>>,
    pub law_candidates: Vec<Located<inferred::law_candidates::Row>>,
    pub law_ordered_terms: Vec<Located<inferred::law_ordered_terms::Row>>,
    pub law_participation_decisions: Vec<Located<inferred::law_participation_decisions::Row>>,
    pub domain_eligible_members: Vec<Located<inferred::domain_eligible_members::Row>>,
    pub ports: Vec<Located<inferred::ports::Row>>,
    pub port_members: Vec<Located<inferred::port_members::Row>>,
    pub port_member_domains: Vec<Located<inferred::port_member_domains::Row>>,
    pub port_state_targets: Vec<Located<inferred::port_state_targets::Row>>,
    pub valid_index_tuples: Vec<Located<inferred::valid_index_tuples::Row>>,
    pub domains: Vec<Located<normalized::domains::Row>>,
    pub domain_members: Vec<Located<normalized::domain_members::Row>>,
    pub domain_products: Vec<Located<normalized::domain_products::Row>>,
    pub connections: Vec<Located<normalized::connections::Row>>,
    pub connection_bindings: Vec<Located<reference::connection_bindings::Row>>,
    pub element_projection_contracts: Vec<Located<reference::element_projection_contracts::Row>>,
}
impl Inventory {
    #[expect(
        clippy::too_many_lines,
        reason = "load keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) async fn load(
        session: &SnapshotSession,
        sources: &Sources,
        ctx: &AlgorithmContext<'_>,
    ) -> Result<(Self, AlgorithmInputs), CompilerError> {
        let mut arguments = AlgorithmInputs::new(ctx.reserver, "P8:typed-law-arguments");
        let inventory = Self {
            law_applications: load::<compiled::law_applications::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            contributions: load::<compiled::contributions::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            expression_root_indices: load::<compiled::expression_root_indices::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            symbol_groups: load::<compiled::symbol_groups::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            symbol_group_members: load::<compiled::symbol_group_members::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            symbols: load::<compiled::symbols::Row>(&mut arguments, session, sources, ctx).await?,
            law_contexts: load::<inferred::law_contexts::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            law_axes: load::<inferred::law_axes::Row>(&mut arguments, session, sources, ctx)
                .await?,
            law_candidates: load::<inferred::law_candidates::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            law_ordered_terms: load::<inferred::law_ordered_terms::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            law_participation_decisions: load::<inferred::law_participation_decisions::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            domain_eligible_members: load::<inferred::domain_eligible_members::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            ports: load::<inferred::ports::Row>(&mut arguments, session, sources, ctx).await?,
            port_members: load::<inferred::port_members::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            port_member_domains: load::<inferred::port_member_domains::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            port_state_targets: load::<inferred::port_state_targets::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            valid_index_tuples: load::<inferred::valid_index_tuples::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            domains: load::<normalized::domains::Row>(&mut arguments, session, sources, ctx)
                .await?,
            domain_members: load::<normalized::domain_members::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            domain_products: load::<normalized::domain_products::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            connections: load::<normalized::connections::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            connection_bindings: load::<reference::connection_bindings::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            element_projection_contracts: load::<reference::element_projection_contracts::Row>(
                &mut arguments,
                session,
                sources,
                ctx,
            )
            .await?,
            session: session.clone(),
            sources: sources.clone(),
        };
        Ok((inventory, arguments))
    }
}
async fn load<T: RelationRow>(
    arguments: &mut AlgorithmInputs,
    session: &SnapshotSession,
    sources: &Sources,
    ctx: &AlgorithmContext<'_>,
) -> Result<Vec<Located<T>>, CompilerError> {
    let spec = T::relation(ctx.registry)?;
    let (port, _) = sources.get(&spec.key).ok_or_else(|| {
        invalid(format!(
            "law source {} has no exact owner",
            spec.key.qualified_name()
        ))
    })?;
    Ok(keyed_rows::<T>(
        arguments,
        scan(session, spec, "argument")?,
        session,
        ctx.registry,
        ctx.cancel,
    )
    .await?
    .into_iter()
    .map(|row| Located {
        row: row.row,
        source: SourceKey {
            relation: spec.key,
            port: port.clone(),
            key: row.key,
        },
    })
    .collect())
}
