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
    model::{InvariantSpec, RelationKey},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

/// The field-directed native literal codec retains actual numeric bits and every nested value.
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
                let rows: Vec<Vec<serde_json::Value>> = values
                    .iter()
                    .map(|row| {
                        assert_eq!(row.len(), spec.columns.len(), "complete row: {name}");
                        spec.columns
                            .iter()
                            .map(|column| {
                                serde_json::from_str(
                                    row.get(column.name()).expect("explicit column"),
                                )
                                .expect("typed literal")
                            })
                            .collect()
                    })
                    .collect();
                let batch =
                    pse_relations::testing::untrusted_batch_from_literals(registry, spec, &rows)
                        .unwrap_or_else(|error| panic!("{name}: {error}"));
                (spec.key, batch)
            })
            .collect()
    }
    pub(crate) fn expected(&self, _registry: &Registry) -> Vec<Vec<serde_json::Value>> {
        self.expected_keys
            .iter()
            .map(|row| {
                row.iter()
                    .map(|value| serde_json::from_str(value).expect("expected typed key"))
                    .collect()
            })
            .collect()
    }
}
pub(crate) fn directory(invariant: &InvariantSpec) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/invariants")
        .join(invariant.qualified_name().replace([':', '.'], "-"))
}
pub(crate) async fn execute(
    registry: &Arc<Registry>,
    fixture: &Fixture,
) -> Vec<Vec<serde_json::Value>> {
    use datafusion::{
        catalog::{CatalogProvider, MemoryCatalogProvider, MemorySchemaProvider, SchemaProvider},
        datasource::MemTable,
        prelude::SessionContext,
    };
    // These are independent oracles for each declared relational query. Negative
    // cases deliberately enter as untrusted Arrow; full product admission is
    // exercised separately and may refuse them before this particular query.
    let invariant = registry
        .invariants()
        .iter()
        .find(|value| value.qualified_name() == fixture.invariant)
        .expect("registered invariant");
    let rows = fixture.batches(registry);
    let factory = pse_testkit::NativeFixture::new(std::num::NonZeroUsize::new(64 << 20).unwrap())
        .unwrap()
        .into_factory();
    let state = factory.native_state().clone();
    let catalog_name = state.config_options().catalog.default_catalog.clone();
    let context = SessionContext::new_with_state(state);
    let catalog = Arc::new(MemoryCatalogProvider::new());
    let mut schemas = BTreeMap::<String, Arc<MemorySchemaProvider>>::new();
    for name in &invariant.inputs {
        let spec = registry.relation(name).expect("declared dependency");
        let batch = rows
            .get(&spec.key)
            .expect("explicit fixture dependency")
            .clone();
        let schema = schemas
            .entry(spec.key.namespace.as_str().to_owned())
            .or_default();
        schema
            .register_table(
                spec.key.name.to_owned(),
                Arc::new(MemTable::try_new(batch.schema(), vec![vec![batch]]).unwrap()),
            )
            .unwrap();
    }
    for (name, schema) in schemas {
        catalog.register_schema(&name, schema).unwrap();
    }
    context.register_catalog(&catalog_name, catalog);
    context
        .sql(&invariant.query)
        .await
        .unwrap_or_else(|error| panic!("{}: {error:?}", fixture.invariant))
        .select(
            invariant
                .key_columns
                .iter()
                .map(|name| datafusion::logical_expr::col(*name)),
        )
        .unwrap()
        .collect()
        .await
        .unwrap()
        .iter()
        .flat_map(|batch| pse_relations::testing::literal_rows(batch).unwrap())
        .collect()
}
