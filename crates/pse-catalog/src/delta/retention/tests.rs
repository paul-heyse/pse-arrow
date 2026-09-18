// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native in-memory reachability query; no IO or durable maintenance.
use super::*;
use crate::session::SessionFactory;
use datafusion::{
    common::ResolvedTableReference,
    execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder},
};
use pse_ids::{FixedBudget, SemanticId};
use pse_relations::generated::enums::{PublicationKind, RetentionReason};
use std::{collections::BTreeMap, sync::Arc};

#[tokio::test]
async fn publication_reachability_projects_typed_distinct_versions_and_empty_lists() {
    let mut builder = pse_schema::RegistryBuilder::new();
    pse_schema::catalog::declare_publications(&mut builder);
    let registry = Arc::new(builder.build().unwrap());
    let spec = publications::spec(&registry).unwrap();
    let key = spec.key;
    let member = publications::RuntimePublicationsFieldMembersItem {
        catalog_name: "unit".into(),
        schema_name: "runtime".into(),
        table_name: "members".into(),
        table_uri: "memory:///member/".into(),
        delta_version: 2,
        relation_id: spec.id,
        relation_version: i64::from(spec.key.version),
        contract_fingerprint: spec.fingerprint,
        selection: publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
    };
    let cancel = CancellationToken::new();
    let factory = SessionFactory::from_builder(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(32 << 20),
        "retention-unit",
        SessionStateBuilder::new_with_default_features(),
    );
    for members in [vec![], vec![member.clone(), member]] {
        let count = usize::from(!members.is_empty());
        let mut rows = publications::Builder::with_registry(&registry, 1).unwrap();
        rows.push(publications::Row {
            workspace_id: SemanticId::from_bytes([1; 16]),
            publication_id: SemanticId::from_bytes([2; 16]),
            parent_publication_id: None,
            attempt_id: SemanticId::from_bytes([3; 16]),
            kind: PublicationKind::Relations,
            inputs: vec![],
            members,
        })
        .unwrap();
        let session = factory
            .candidate_checked(
                BTreeMap::from([(key, rows.finish().unwrap())]),
                registry.clone(),
                &cancel,
            )
            .unwrap();
        let source = session.table_reference(&key).unwrap();
        let input = session
            .relation_plan(&ResolvedTableReference {
                catalog: source.catalog().unwrap().into(),
                schema: source.schema().unwrap().into(),
                table: source.table().into(),
            })
            .unwrap();
        let retention = session.publication_retention(&input, &cancel).unwrap();
        let retention = session
            .combine_retention(&[retention.clone(), retention], &cancel)
            .unwrap();
        let result = session
            .prepare(retention.plan().clone(), &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap()
            .into_batches();
        let mut actual = vec![];
        for batch in result {
            let view =
                retained_versions::View::try_from_batch_with_registry(&registry, &batch).unwrap();
            for index in 0..batch.num_rows() {
                actual.push(view.row(index).unwrap());
            }
        }
        assert_eq!(actual.len(), count);
        if count != 0 {
            assert_eq!(
                actual[0],
                retained_versions::Row {
                    table_uri: "memory:///member/".into(),
                    from_version: 2,
                    through_version: 2,
                    reason: RetentionReason::Publication
                }
            );
        }
    }
}
