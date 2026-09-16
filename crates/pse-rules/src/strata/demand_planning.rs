// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Demand and law declarations prepare native plans for every truth branch.
use crate::plan::{PortBinding, compile};
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

#[test]
fn every_demand_and_law_truth_candidate_prepares_with_its_exact_input_fields() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let rules = registry
        .rules()
        .iter()
        .filter(|rule| rule.name.starts_with("P6.") || rule.name.starts_with("P8."))
        .collect::<Vec<_>>();
    assert!(!rules.is_empty());
    let mut inputs = BTreeMap::new();
    for rule in &rules {
        for (name, _, _) in rule.plan.dependencies() {
            let spec = registry.relation(name).unwrap();
            inputs
                .entry(spec.key)
                .or_insert_with(|| FieldCheckedBatch::concat(&registry, spec, &[]).unwrap());
        }
    }
    let one = NonZeroUsize::new(1).unwrap();
    let session = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(256 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        },
        native_engine_profile(),
    )
    .unwrap()
    .candidate_checked(inputs, registry.clone(), &CancellationToken::new())
    .unwrap();
    let mut failures = Vec::new();
    for rule in rules {
        let binding = PortBinding {
            ports: rule
                .plan
                .dependencies()
                .into_iter()
                .map(|(name, port, _)| (port.to_owned(), registry.relation(name).unwrap().key))
                .collect(),
        };
        for (truth, plan) in crate::plan::outcomes::candidates(&rule.plan) {
            let mut candidate = rule.clone();
            candidate.plan = plan;
            let prepared =
                compile(&candidate, &binding, &session, &registry).and_then(|compiled| {
                    Ok(session.prepare_rule_plan(compiled.plan, &CancellationToken::new())?)
                });
            if let Err(error) = prepared {
                failures.push(format!("{} ({truth}): {error}", rule.name));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} native planning failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
