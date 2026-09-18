// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite positive fixed points over admitted Arrow workspaces (blueprint §6.15.2).
mod admission;
pub mod completed;
#[cfg(test)]
mod demand_planning;
pub mod native;
pub mod native_input;
mod native_materialize;
mod native_state;
mod native_support;
pub(crate) mod relational;
mod rounds;

use crate::{RuleError, errmap::internal};
use datafusion::arrow::array::RecordBatch;
use native_state::State;
use pse_catalog::session::{PlanObservation, SnapshotSession};
use pse_ids::{CancellationToken, SemanticId};
use pse_schema::{
    Registry,
    model::{RelationKey, RuleSpec},
};
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Location of a complete rule input. Actual owned fields establish membership;
/// optional typed Delta selections retain its exact durable source.
#[derive(Clone, Debug)]
pub enum RuleInputLocation {
    /// Owned admitted fields with an exact Delta selection when durably sourced.
    /// The owner has no snapshot, predecessor graph or producing computation.
    Facts(Arc<pse_catalog::session::RelationFacts>),
    /// A declared finite workspace, without a fabricated snapshot identity.
    Workspace,
    /// An exact member of an opaque completed rule program; only the executor mints it.
    Completed(Arc<completed::CompletedRelation>),
    /// Actual native completion with validated typed source witness keys.
    Native(Arc<native_input::NativeInput>),
}
/// The exact relation and source receipt chosen for one input port.
#[derive(Clone, Debug)]
pub struct LocatedRuleInput {
    /// Declared schema version, never inferred from values.
    pub relation: RelationKey,
    /// Complete source binding supplied by the admitted pass context.
    pub location: RuleInputLocation,
}
/// Named bindings for one rule. Reads cannot fall back to an ambient latest table.
#[derive(Clone, Debug, Default)]
pub struct RuleBindings {
    /// One explicit input per declared port, including workspace scans.
    pub ports: BTreeMap<String, LocatedRuleInput>,
}
/// Execution guards bound unfinished work; exhaustion is an error, never convergence.
#[derive(Clone, Copy, Debug)]
pub struct StratumLimits {
    /// Maximum rounds per stratum. Finite-domain checks are independent of this guard.
    pub max_rounds: NonZeroU32,
}
/// Fully settled heads, typed assertions, outcomes and located finite support.
#[derive(Debug)]
pub struct StratumOutcome {
    /// Immutable whole-program receipt, sealed before exposing the result map.
    pub completed: Arc<completed::CompletedProgram>,
    /// Exact declared relation map. No partial map is returned on failure.
    pub relations: BTreeMap<RelationKey, RecordBatch>,
    /// Declared provenance sidecars, routed through the driver attempt lifecycle.
    pub derivations: Vec<RecordBatch>,
    /// Engine observations for the actual candidate and support queries.
    pub plans: Vec<PlanObservation>,
    /// Actual rounds completed per stratum.
    pub rounds: BTreeMap<u16, u32>,
}

/// Execute complete producer sets in stratum order using the shared sealed engine.
///
/// # Errors
/// Invalid/partial bindings, non-finite recursive plans, conflicts under Reject,
/// unfinished rounds, invalid head values, cancellation or exhausted resources.
pub async fn execute_strata(
    rules: &[RuleSpec],
    bindings: &BTreeMap<SemanticId, RuleBindings>,
    session: &SnapshotSession,
    registry: &Registry,
    outputs: &BTreeSet<RelationKey>,
    limits: StratumLimits,
    cancel: &CancellationToken,
) -> Result<StratumOutcome, RuleError> {
    let (session, plan) =
        native::plan_strata(rules, bindings, session, registry, outputs, limits, cancel)?;
    let datafusion_expr::LogicalPlan::Extension(extension) = plan else {
        return Err(internal(
            "finite program did not produce its native extension",
        ));
    };
    let node = extension
        .node
        .as_any()
        .downcast_ref::<native::FixedPoint>()
        .ok_or_else(|| internal("finite program produced a different extension"))?;
    native::execute(node, &session, cancel).await
}

async fn iterate(
    rules: &[RuleSpec],
    bindings: &BTreeMap<SemanticId, RuleBindings>,
    session: &SnapshotSession,
    registry: &Registry,
    outputs: &BTreeSet<RelationKey>,
    limits: StratumLimits,
    cancel: &CancellationToken,
) -> Result<native::Settled, RuleError> {
    admission::validate(rules, bindings, session, registry, outputs, cancel)?;
    let mut state = State::new(registry, rules)?;
    validate_native_mappings(&mut state, bindings, session, cancel).await?;
    let execution = session;
    let mut rounds = BTreeMap::new();
    let strata = rules
        .iter()
        .map(|rule| rule.stratum)
        .collect::<BTreeSet<_>>();
    for stratum in strata {
        let active = rules
            .iter()
            .filter(|rule| rule.stratum == stratum)
            .collect::<Vec<_>>();
        let started = std::time::Instant::now();
        let mut program = Box::pin(rounds::Program::prepare(
            &active, bindings, &state, execution, registry, cancel,
        ))
        .await?;
        tracing::info!(
            stratum,
            rules = active.len(),
            elapsed_seconds = started.elapsed().as_secs_f64(),
            "finite rule stratum prepared"
        );
        let mut settled = false;
        for ordinal in 1..=limits.max_rounds.get() {
            cancel
                .checkpoint()
                .map_err(pse_catalog::CatalogError::from)?;
            let started = std::time::Instant::now();
            tracing::info!(stratum, round = ordinal, "finite rule round started");
            let changed = program
                .execute(&mut state, registry, cancel, ordinal)
                .await?;
            tracing::info!(
                stratum,
                round = ordinal,
                changed,
                elapsed_seconds = started.elapsed().as_secs_f64(),
                "finite rule round completed"
            );
            rounds.insert(stratum, ordinal);
            if !changed {
                settled = true;
                break;
            }
        }
        if !settled {
            return Err(RuleError::ResourceLimit {
                consumer: format!("rule stratum {stratum} has unfinished finite work"),
                config_keys: vec!["pse.rules.max_rounds".to_owned()],
            });
        }
        state
            .complete_stratum(&active, registry, execution, cancel)
            .await?;
    }
    let (relations, derivations) = state.materialize(registry, execution, cancel).await?;
    Ok(native::Settled {
        relations,
        derivations,
        rounds,
    })
}

async fn validate_native_mappings(
    state: &mut State,
    bindings: &BTreeMap<SemanticId, RuleBindings>,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<(), RuleError> {
    let mut roles = BTreeMap::new();
    for (rule, binding) in bindings {
        for (port, input) in &binding.ports {
            if let RuleInputLocation::Native(input) = &input.location {
                roles.insert(
                    native_support::mapping_role(*rule, port),
                    input.support_mapping().clone(),
                );
            }
        }
    }
    for name in roles.keys() {
        if !state
            .check_empty(
                relational::identity_collisions(session.scan_role(name)?, "mapping_id")?,
                session,
                cancel,
            )
            .await?
        {
            return Err(internal(
                "distinct actual native witnesses share one mapping label",
            ));
        }
    }
    Ok(())
}
