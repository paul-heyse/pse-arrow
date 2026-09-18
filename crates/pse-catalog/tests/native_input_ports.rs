// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native port binding retains exact roles, versions and immutable providers.

use datafusion::{
    arrow::array::{RecordBatch, UInt64Array},
    datasource::source_as_provider,
    execution::runtime_env::RuntimeEnv,
};
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    Registry, RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::{collections::BTreeMap, sync::Arc};

#[expect(clippy::unwrap_used, reason = "fixed versioned registry fixture")]
fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    for (name, version) in [("values", 1), ("values", 2), ("unique", 1)] {
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                name,
                version,
                Authority::Authored,
                SnapshotClass::Model,
                "Actual native port values.",
            )
            .pk(&["id"])
            .columns(vec![FieldContract::key(
                "id",
                FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
                "Value.",
            )]),
        );
    }
    Arc::new(builder.build().unwrap())
}

#[tokio::test]
async fn named_ports_keep_versions_distinct_and_alias_only_unique_provider_names() {
    let registry = registry();
    let batch = |name, version, values: Vec<u64>| {
        let spec = registry
            .relations()
            .iter()
            .find(|spec| spec.key.name == name && spec.key.version == version)
            .unwrap();
        let batch = RecordBatch::try_new(
            Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap()),
            vec![Arc::new(UInt64Array::from(values))],
        )
        .unwrap();
        FieldCheckedBatch::admit(&registry, spec, batch).unwrap()
    };
    let factory = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 2.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap();
    let cancel = CancellationToken::new();
    for version in [1, 2] {
        let session = factory
            .candidate_checked_ports(
                BTreeMap::from([
                    ("before".into(), batch("values", 1, vec![7])),
                    ("after".into(), batch("values", version, vec![11])),
                    ("empty".into(), batch("unique", 1, vec![])),
                ]),
                Arc::clone(&registry),
                &cancel,
            )
            .unwrap();
        let sql_before = session
            .sql("SELECT id FROM roles.inputs.before", &cancel)
            .await
            .unwrap();
        assert_eq!(
            sql_before.iter().map(RecordBatch::num_rows).sum::<usize>(),
            1
        );
        assert_eq!(session.input_roles().count(), 3);
        for spec in registry
            .relations()
            .iter()
            .filter(|spec| spec.key.name == "values")
        {
            assert!(session.table_source(&spec.key).is_err());
        }
        let unique = registry.relation("authored.unique").unwrap();
        let role = source_as_provider(&session.role_source("empty").unwrap()).unwrap();
        assert!(Arc::ptr_eq(
            &session.table_provider(&unique.key).unwrap(),
            &role
        ));
        for (role, expected) in [("before", vec![7]), ("after", vec![11]), ("empty", vec![])] {
            let complete = session
                .prepare(session.scan_role(role).unwrap(), &cancel)
                .unwrap()
                .execute(&cancel)
                .await
                .unwrap();
            let values = complete
                .batches()
                .iter()
                .flat_map(|batch| {
                    batch
                        .column(0)
                        .as_any()
                        .downcast_ref::<UInt64Array>()
                        .unwrap()
                        .values()
                        .to_vec()
                })
                .collect::<Vec<_>>();
            assert_eq!(values, expected);
        }
        assert!(session.scan_role("absent").is_err());
    }
}
