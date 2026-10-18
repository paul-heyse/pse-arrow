// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual source-publication receipts over recorded admitted physical row blocks.
#![allow(clippy::unwrap_used, reason = "bounded publication fixtures require exact successful setup")]
use super::*;
use super::super::physical_spool::{PhysicalGroups, PhysicalSpool, PhysicalSpoolReader};
use std::collections::BTreeMap;

fn identity(relation: pse_schema::model::RelationKey, payloads: &[Vec<u8>]) -> ContentHash {
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("physical.rows.v1").str(&relation.to_string());
    for payload in payloads { hash.part(payload); }
    hash.finish_hash()
}
fn reader(runtime: &super::super::Runtime, payloads: &[Vec<u8>], memory: usize) -> PhysicalSpoolReader {
    let mut spool = PhysicalSpool::new(runtime.shared.runtime_env(),memory).unwrap();
    for payload in payloads { spool.push(payload).unwrap(); }
    spool.finish().unwrap()
}
async fn encoded(table: &pse_relations::columnar::FieldCheckedBatch) -> Vec<Vec<u8>> {
    let mut payloads = Vec::new();
    super::super::result_blocks::visit_source_blocks_async(table.batch(), |_,_,payload| {
        payloads.push(payload); std::future::ready(Ok::<_,WorkflowError>(()))
    }).await.unwrap();
    payloads
}
async fn exact_rows(store: &CanonicalStore, revision: &Revision, expected: &[Vec<u8>]) {
    let pin = store.protect(revision.clone(),Duration::from_secs(60)).await.unwrap();
    let mut actual = BTreeMap::new(); let mut after = String::new();
    loop {
        let members = store.membership_page(&pin,None,&after).await.unwrap();
        if members.is_empty() { break; }
        after = members.last().unwrap().key.clone();
        for member in members {
            let batch = store.selected_object_batch(&pin,std::slice::from_ref(&member.version)).await.unwrap();
            let length = batch.extents().next().unwrap().1;
            let mut payload = Vec::with_capacity(length);
            while payload.len()<length {
                let blocks = store.selected_object_blocks(&batch,0,(payload.len()/pse_operations::canonical_staging::SOURCE_BLOCK_BYTES) as u64).await.unwrap();
                assert!(!blocks.is_empty());
                for (object,block) in blocks {
                    assert_eq!(object,0);
                    assert_eq!(block.ordinal as usize*pse_operations::canonical_staging::SOURCE_BLOCK_BYTES,payload.len());
                    payload.extend_from_slice(block.payload.as_slice());
                }
            }
            let received = store.finish_selected_object_batch(batch,vec![payload]).await.unwrap();
            assert_eq!(received.len(),1);
            assert_eq!(received[0].kind,"physical:rows:v1");
            actual.insert(member.logical,received.into_iter().next().unwrap().payload);
        }
    }
    store.release(&pin).await.unwrap();
    assert_eq!(actual.len(),expected.len());
    for (ordinal,payload) in expected.iter().enumerate() {
        assert_eq!(actual[&format!("rows:{ordinal:020}")].as_slice(),payload);
    }
}

#[tokio::test]
async fn canonical_physical_empty_source_map_publishes_accounted_context_receipt() {
    let runtime = super::super::durable_tests::durable_runtime();
    let operations = runtime.operations().unwrap();
    let (documents,_) = super::super::worker_tests::sources("package p { def Root { param x:Scalar=3; annotation report x(\"constant\"); } }");
    let source = operations.put_sources(&documents).await.unwrap();
    let mut physical = runtime.physical_source(&source,&crate::CancelSource::new()).await.unwrap();
    physical.sources.clear();
    let context = format!("physical:context:{}",physical.identity());
    assert!(operations.store.revision(&context).await.unwrap().is_none());
    let constrained = Operations {pool: Arc::new(datafusion::execution::memory_pool::GreedyMemoryPool::new(1<<20)),..operations.clone()};
    assert!(super::super::result_projection::projection_memory_refusal(
        &constrained.put_physical_rows(&runtime,&physical).await.unwrap_err()));
    assert_eq!(constrained.pool.reserved(),0);
    assert!(operations.store.revision(&context).await.unwrap().is_none(),"context grant refusal precedes publication");
    let rows = operations.put_physical_rows(&runtime,&physical).await.unwrap();
    assert_eq!(rows.revisions.len(),1); assert_eq!(rows.revisions[0].key,context);
    assert_eq!(rows.revisions[0].sequence,1);
    assert_eq!(operations.put_physical_rows(&runtime,&physical).await.unwrap().revisions,rows.revisions);
}

#[tokio::test]
async fn canonical_physical_grouped_publication_preserves_exact_replay_and_relation_receipts() {
    let runtime = super::super::durable_tests::durable_runtime();
    let operations = runtime.operations().unwrap();
    let (documents,_) = super::super::worker_tests::sources("package p { def Root { param x:Scalar=3; annotation report x(\"constant\"); } }");
    let source = operations.put_sources(&documents).await.unwrap();
    let mut physical = runtime.physical_source(&source,&crate::CancelSource::new()).await.unwrap();
    let baseline = operations.put_physical_rows(&runtime,&physical).await.unwrap();
    let (&relation,table) = physical.sources.iter().filter(|(_,table)|table.batch().num_rows()>0)
        .min_by_key(|(_,table)|table.batch().get_array_memory_size()/table.batch().num_rows()).unwrap();
    let old_identity = identity(relation,&encoded(table).await);
    let seed = table.slice(0,1).unwrap();
    // Repeated admitted rows exercise transport and exact retained bytes. They
    // are deliberately not compiled or claimed as a valid scientific context.
    let tall = seed.take_reserved(&datafusion::arrow::array::UInt32Array::from(vec![0;32769]),
        &runtime.shared.pool(),&pse_columnar::CancellationToken::new()).unwrap();
    let payloads = encoded(&tall).await;
    assert!((2..=4).contains(&payloads.len()));
    assert!(payloads.iter().map(Vec::len).sum::<usize>()<=2*pse_operations::canonical::PAYLOAD_BYTES);
    let content = identity(relation,&payloads);
    // Old per-block records coexist without adopting their head or changing any
    // old request's meaning. This fixture alone creates the historical format.
    let mut old = None::<Revision>;
    for (ordinal,payload) in payloads.iter().enumerate() {
        let edit = source_edit(format!("rows:{ordinal:020}"),relation.to_string(),format!("{ordinal:020}"),"physical:rows:v1",payload.clone());
        old = Some(operations.store.edit(&format!("physical:rows:{content}"),old.as_ref().map(|row|row.key.as_str()),
            &format!("physical:{content}:{ordinal}"),&[edit]).await.unwrap());
    }
    let old = old.unwrap();
    physical.sources.insert(relation,tall);
    let published = operations.put_physical_rows(&runtime,&physical).await.unwrap();
    let fitting = published.revisions.iter().find(|row|row.problem==format!("physical:rows:grouped-v2:{content}")).unwrap();
    assert_eq!(fitting.sequence,1); assert_eq!(fitting.parent,None);
    assert_eq!(fitting.key,format!("physical:grouped-v2:{content}:{:020}:{:020}",0,payloads.len()));
    exact_rows(&operations.store,fitting,&payloads).await;
    assert_eq!(operations.store.revision(&old.key).await.unwrap(),Some(old.clone()));
    exact_rows(&operations.store,&old,&payloads).await;
    let replaced = format!("physical:rows:grouped-v2:{old_identity}");
    for original in &baseline.revisions {
        if original.problem!=replaced { assert!(published.revisions.contains(original),"other relation/context receipts retain exact identity"); }
    }
    runtime.clear_program_cache();
    let replayed = operations.put_physical_rows(&runtime,&physical).await.unwrap();
    assert_eq!(replayed.revisions,published.revisions,"cold exact replay returns original whole receipts");

    let encoding = MemoryConsumer::new("test:physical-publisher-encoder-sdk").register(&operations.pool);
    encoding.try_grow(16*pse_operations::canonical::PAYLOAD_BYTES).unwrap();
    pse_operations::canonical::response_ownership_scope(pse_columnar::AllocationLease::new(encoding),async {
    // Five known one-row original IPC blocks isolate the count boundary for
    // interruption/replay. The large fitting relation above exercises real
    // payload publication; this fault fixture makes no throughput claim.
    let one = encoded(&seed).await;
    assert_eq!(one.len(),1);
    let many = vec![one.into_iter().next().unwrap();5];
    let content = identity(relation,&many);
    let mut groups = PhysicalGroups::new(reader(&runtime,&many,0),&runtime.shared.pool()).unwrap();
    let mut first = groups.next().await.unwrap().unwrap();
    assert_eq!((first.first,first.end),(0,4));
    let original_ack = operations.publish_physical_group(relation,content,None,&mut first).await.unwrap();
    // The first group committed successfully. Discard this caller's continuation
    // before any next group effect; this is interruption/replay, not network loss.
    drop(first); drop(groups);
    let mut replay = PhysicalGroups::new(reader(&runtime,&many,many.iter().map(|payload|payload.len()+8).sum()),&runtime.shared.pool()).unwrap();
    let mut same = replay.next().await.unwrap().unwrap();
    assert_eq!(operations.publish_physical_group(relation,content,None,&mut same).await.unwrap(),original_ack);
    drop(same); drop(replay);
    let final_ack = operations.publish_physical_groups(relation,content,reader(&runtime,&many,0)).await.unwrap().unwrap();
    assert_eq!(final_ack.sequence,2); assert_eq!(final_ack.parent.as_ref(),Some(&original_ack.key));
    assert_eq!(operations.store.revision(&original_ack.key).await.unwrap(),Some(original_ack.clone()));
    exact_rows(&operations.store,&final_ack,&many).await;
    assert_eq!(operations.publish_physical_groups(relation,content,reader(&runtime,&many,1024)).await.unwrap(),Some(final_ack));

    for changed_bytes in [false,true] {
        let mut altered = PhysicalGroups::new(reader(&runtime,&many,0),&runtime.shared.pool()).unwrap();
        let mut group = altered.next().await.unwrap().unwrap();
        if changed_bytes { group.blocks[0][0]^=1; } else { group.blocks.pop(); }
        assert!(matches!(operations.publish_physical_group(relation,content,None,&mut group).await,
            Err(WorkflowError::Canonical(pse_operations::canonical::CanonicalError::OperationReused))));
    }
    assert_eq!(operations.store.revision(&original_ack.key).await.unwrap(),Some(original_ack));
    }).await;
}
