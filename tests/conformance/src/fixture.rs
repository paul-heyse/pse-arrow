// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual typed inputs and exact expected keys for each declared invariant.
#![allow(
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "shared deterministic fixture helpers are used by separate test/example binaries"
)]

#[path = "fixture/generate.rs"]
mod generate;
use datafusion::arrow::array::RecordBatch;
pub(crate) fn pair(registry: &Registry, invariant: &InvariantSpec) -> (Fixture, Fixture) {
    generate::pair(registry, invariant)
}
use pse_schema::{
    Registry,
    model::{Cell, InvariantSpec, RelationKey},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

/// The tagged Cell codec retains actual numeric bits and every nested value.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Fixture {
    pub invariant: String,
    pub rows: BTreeMap<String, Vec<BTreeMap<String, String>>>,
    pub expected_keys: Vec<Vec<String>>,
}
impl Fixture {
    pub(crate) fn load(path: &Path) -> Self {
        serde_saphyr::from_str(&std::fs::read_to_string(path).expect("read fixture"))
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    }
    pub(crate) fn batches(&self, registry: &Registry) -> BTreeMap<RelationKey, RecordBatch> {
        self.rows
            .iter()
            .map(|(name, values)| {
                let spec = registry.relation(name).expect("declared fixture relation");
                let rows: Vec<Vec<Cell>> = values
                    .iter()
                    .map(|row| {
                        assert_eq!(row.len(), spec.columns.len(), "complete row: {name}");
                        spec.columns
                            .iter()
                            .map(|column| {
                                Cell::from_literal_spec(
                                    row.get(column.name()).expect("explicit column"),
                                    registry,
                                )
                                .expect("typed literal")
                            })
                            .collect()
                    })
                    .collect();
                let batch = pse_relations::cells::batch_from_cells(registry, spec, &rows)
                    .unwrap_or_else(|error| panic!("{name}: {error}"));
                (spec.key, batch)
            })
            .collect()
    }
    pub(crate) fn expected(&self, registry: &Registry) -> Vec<Vec<Cell>> {
        self.expected_keys
            .iter()
            .map(|row| {
                row.iter()
                    .map(|value| {
                        Cell::from_literal_spec(value, registry).expect("expected typed key")
                    })
                    .collect()
            })
            .collect()
    }
    pub(crate) fn save(&self, path: &Path) {
        std::fs::create_dir_all(path.parent().expect("fixture directory"))
            .expect("create directory");
        let text = serde_json::to_string_pretty(self).expect("YAML 1.2 JSON subset");
        std::fs::write(path, format!("{text}\n")).expect("write fixture");
    }
}
pub(crate) fn directory(invariant: &InvariantSpec) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/invariants")
        .join(invariant.qualified_name().replace([':', '.'], "-"))
}
pub(crate) async fn execute(registry: &Arc<Registry>, fixture: &Fixture) -> Vec<Vec<Cell>> {
    use pse_catalog::session::{
        ExecutionSettings, ThreadBudget, build_candidate_session, native_engine_profile,
    };
    use pse_ids::{CancellationToken, FixedBudget};
    use std::num::NonZeroUsize;
    let invariant = registry
        .invariants()
        .iter()
        .find(|value| value.qualified_name() == fixture.invariant)
        .expect("registered invariant");
    let rule = registry.rule(&invariant.rule).expect("declared rule");
    let rows = fixture.batches(registry);
    let mut ports = pse_rules::plan::PortBinding::default();
    for (relation, port, _) in rule.plan.dependencies() {
        let key = registry
            .relation(relation)
            .expect("declared dependency")
            .key;
        assert!(
            rows.contains_key(&key),
            "explicit fixture dependency {relation}"
        );
        ports.ports.insert(port.to_owned(), key);
    }
    let thread = NonZeroUsize::new(1).unwrap();
    let session = build_candidate_session(
        rows,
        Arc::clone(registry),
        Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: thread,
            target_partitions: thread,
        },
        native_engine_profile(),
    )
    .expect("candidate session without constraints");
    let plan =
        pse_rules::plan::compile(rule, &ports, &session, registry).expect("compile fixture rule");
    let result = pse_rules::exec::execute(&plan, &session, registry, &CancellationToken::default())
        .await
        .unwrap_or_else(|error| panic!("{}: {error:?}", fixture.invariant));
    assert!(
        result.undecided.iter().all(|batch| batch.num_rows() == 0),
        "fixture is decided"
    );
    result
        .head
        .iter()
        .flat_map(|batch| {
            pse_relations::cells::decode_columns(registry, batch).expect("actual typed result keys")
        })
        .collect()
}
