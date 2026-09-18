// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed component inputs for native plan tests. Workspace locations never mint
//! producer evidence or claim that an upstream compiler stage ran.
use super::native_outputs::Sources;
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, SnapshotSession, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_relations::columnar::{FieldCheckedBatch, RelationRow};
use pse_rules::strata::{LocatedRuleInput, RuleInputLocation};
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

pub(super) type Inputs = BTreeMap<RelationKey, FieldCheckedBatch>;

pub(super) fn put<T: RelationRow>(rows: &mut Inputs, registry: &Registry, values: Vec<T>) {
    let mut builder = T::builder(registry, values.len()).unwrap();
    for row in values {
        T::push(&mut builder, row).unwrap();
    }
    rows.insert(
        T::relation(registry).unwrap().key,
        T::finish(builder).unwrap(),
    );
}

pub(super) fn read<T: RelationRow>(rows: &Inputs, registry: &Registry) -> Vec<T> {
    T::rows(&rows[&T::relation(registry).unwrap().key]).unwrap()
}

pub(crate) fn session(
    registry: &Arc<Registry>,
    inputs: Inputs,
    budget: &Arc<FixedBudget>,
    cancel: &CancellationToken,
) -> Result<(SnapshotSession, Sources), crate::CompilerError> {
    let mut sources = Sources::new();
    for &relation in inputs.keys() {
        sources.insert(
            relation,
            (
                relation.qualified_name(),
                LocatedRuleInput {
                    relation,
                    location: RuleInputLocation::Workspace,
                },
            ),
        );
    }
    let one = NonZeroUsize::new(1).unwrap();
    let session = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        budget.clone(),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        },
        native_engine_profile(),
    )?
    .with_query_planner(crate::query_planner())
    .candidate_checked(inputs, Arc::clone(registry), cancel)?;
    Ok((session, sources))
}
