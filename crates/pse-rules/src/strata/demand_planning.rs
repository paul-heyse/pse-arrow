// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Demand and law declarations prepare native plans for every truth branch.
use crate::plan::{PortBinding, compile_query};
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

#[tokio::test]
async fn every_native_rule_binds_with_its_exact_input_fields() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let rules = registry.rules().iter().collect::<Vec<_>>();
    assert!(!rules.is_empty());
    let mut inputs = BTreeMap::new();
    for rule in &rules {
        for input in &rule.inputs {
            let name = input.relation.as_str();
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
                .inputs
                .iter()
                .map(|input| {
                    (
                        input.port.to_owned(),
                        registry.relation(&input.relation).unwrap().key,
                    )
                })
                .collect(),
        };
        for query in &rule.queries {
            let truth = query.truth;
            let prepared = compile_query(
                rule,
                query,
                &binding,
                &session,
                &registry,
                &CancellationToken::new(),
            )
            .await
            .and_then(|compiled| {
                crate::plan::trace::compile_support(
                    &compiled.plan,
                    rule,
                    &registry,
                    &session,
                    &CancellationToken::new(),
                )?;
                Ok(())
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
