// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    reason = "test fixture construction and exact independent value assertions"
)]
//! Fresh declared native values, native writes and exact cold publication readers.
#![allow(
    dead_code,
    reason = "shared fixture constructors differ between test binaries"
)]

use datafusion::{
    arrow::array::{Int64Array, RecordBatch},
    common::ResolvedTableReference,
};
use pse_catalog::{
    artifact::{ArtifactPlan, PublicationTarget, RelationOutput},
    delta::publication::{Publication, PublicationRoot},
};
use pse_columnar::CancellationToken;
use pse_relations::generated::{enums::PublicationKind, runtime::publications};
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, sync::Arc};
pub(crate) fn name(schema: &str, table: &str) -> ResolvedTableReference {
    ResolvedTableReference {
        catalog: "artifact".into(),
        schema: schema.into(),
        table: table.into(),
    }
}
pub(crate) async fn publish(
    registry: Arc<Registry>,
    batches: BTreeMap<RelationKey, RecordBatch>,
) -> (
    Publication,
    tempfile::TempDir,
    Arc<dyn pse_columnar::MemoryPool>,
) {
    let fixture = pse_testkit::NativeFixture::new((128 << 20).try_into().unwrap()).unwrap();
    let pool = fixture.resources.pool.clone();
    let factory = Arc::new(fixture.into_factory().with_query_planner(Arc::new(
        pse_engine::session::planner::UnifiedPlanner::new(pse_catalog::assembly::planners()),
    )));
    let cancel = CancellationToken::new();
    let session = factory
        .candidate(batches, Arc::clone(&registry), &cancel)
        .unwrap();
    let outputs = session
        .input_keys()
        .map(|key| {
            let reference = session.table_reference(&key).unwrap();
            let source = session
                .relation_plan(&ResolvedTableReference {
                    catalog: reference.catalog().unwrap().into(),
                    schema: reference.schema().unwrap().into(),
                    table: reference.table().into(),
                })
                .unwrap();
            (
                name(key.namespace.as_str(), key.name),
                RelationOutput {
                    relation_id: source.relation_id(),
                    plan: source.plan().clone(),
                },
            )
        })
        .collect();
    let artifact = ArtifactPlan::new(session, outputs, &cancel).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let base = url::Url::from_directory_path(directory.path()).unwrap();
    let control = base.join("control/").unwrap();
    let destinations = artifact
        .outputs()
        .keys()
        .enumerate()
        .map(|(index, name)| {
            (
                name.clone(),
                base.join(&format!("members/{index}/")).unwrap(),
            )
        })
        .collect();
    let header = publications::Row {
        workspace_id: pse_authoring::ids::uuid_v7(),
        publication_id: pse_authoring::ids::uuid_v7(),
        parent_publication_id: None,
        attempt_id: pse_authoring::ids::uuid_v7(),
        kind: PublicationKind::Relations,
        inputs: vec![],
        members: vec![],
    };
    let command = artifact
        .prepare_publication(
            PublicationTarget {
                reference: name("runtime", "publications"),
                location: control.clone(),
            },
            header,
            destinations,
            vec![],
            &cancel,
        )
        .map(|(command, _ticket)| command)
        .unwrap();
    let result = command.execute(&cancel).await.unwrap();
    let version = result.batches()[0]
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    drop(result);
    drop(artifact);
    // Keep the shared fixture's stack bounded as member opens gain concurrency.
    let publication = Box::pin(Publication::open(
        PublicationRoot {
            location: control,
            version,
        },
        registry,
        &factory,
        &cancel,
    ))
    .await
    .unwrap();
    (publication, directory, pool)
}
