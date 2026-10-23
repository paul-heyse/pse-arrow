// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public workbench/observer controls over small original sealed storage fixtures.
#![allow(unsafe_code,reason="isolated typed result fixtures explicitly own their scientific observation assertions")]
use super::*;
use crate::workflow::{self as flow,StudyObservationRequest,StudyObservationContinuity};
use pse_operations::{canonical::Revision,canonical_execution::{RunRequest,TerminalClass,SeedEligibility,ResultPayload},canonical_studies::{NewOccurrence,point_key,point_facts}};
use pse_model::{study::{PointPolicy,OccurrenceKey,StartPolicy,SeedNeed,PointOutcome,ScientificFacts},generated::enums::StudyPointState};
use pse_relations::generated::runtime::simulation_samples;
use std::time::Duration;
fn request(revision:&Revision,physical:&Revision,key:&str)->RunRequest {RunRequest{key:key.into(),revision:revision.clone(),sources:vec![physical.clone()],request:vec![1],source_selection:vec![2],attestation:vec![3]}}
async fn create(runtime:&Runtime,study:&str,source:&Revision,physical:&flow::PhysicalSource,points:u32) {
    let store=runtime.canonical_store().unwrap();
    let physical_row=store.revision(&physical.revision).await.unwrap().unwrap();
    let points=(0..points).map(|key|NewOccurrence{policy:PointPolicy{key:OccurrenceKey(key),dependencies:vec![],start:StartPolicy::Fresh,seed_need:SeedNeed::NotNeeded,attempt_limit:1},descriptor:vec![42],run:request(source,&physical_row,&format!("{study}-point-{key}"))}).collect::<Vec<_>>();
    let metadata=serde_json::to_vec(&serde_json::json!({"version":4,"physical":physical,"context":null,"variants":null,"diagnostics":null})).unwrap();
    store.create_study(study,&request(source,&physical_row,&format!("{study}-parent")),&metadata,&points,&||false).await.unwrap();
}
async fn finish(runtime:&Runtime,study:&str,key:u32,quantity:pse_ids::SemanticId,unit:pse_ids::SemanticId,owner:pse_ids::SemanticId,value:Option<f64>) {
    let store=runtime.canonical_store().unwrap();let point=point_key(study,OccurrenceKey(key));
    let scope=store.study_scope(&point).await.unwrap();
    let claim=store.claim_study_point(&scope,None,&format!("claim-{point}"),"worker",Duration::from_secs(60)).await.unwrap();
    if let Some(value)=value {
        let mut samples=simulation_samples::Builder::with_registry(runtime.registry(),3,&runtime.validation_context()).unwrap();
        let run_id=pse_operations::mint_id();
        for sample in 0..3 {samples.push(simulation_samples::Row{run_id,sample,time:sample as f64,symbol_id:owner,quantity_id:quantity,unit_id:unit,value:value+sample as f64*2.0}).unwrap();}
        flow::result_projection::store_result_table(store,&claim.fence,simulation_samples::RELATION_ID,flow::ResultCursor::from_checked(samples.finish().unwrap()),&runtime.shared.pool(),&flow::durable::PublicationAcks::new(&runtime.shared.pool())).await.unwrap();
        // An unrelated payload deliberately is not scientific IPC. Exact selected detail
        // must never decode it to open the requested original sample relation.
        store.append_result_batch(&claim.fence,&format!("opaque-{point}"),"unrelated-opaque-fixture",0,ResultPayload::new(vec![255,254]).unwrap(),1).await.unwrap();
    }
    let closed=store.close_result_ingestion(&claim.fence,&format!("close-{point}")).await.unwrap();
    let manifest=store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: these small sealed typed samples and finite values are fixture-owned;
    // this control tests storage composition, not solver or scientific qualification.
    unsafe {store.seal_attempt(&manifest,&format!("seal-{point}"),if value.is_some(){TerminalClass::Succeeded}else{TerminalClass::Failed},&[],SeedEligibility::Unavailable).await}.unwrap();
    let mut facts=point_facts(&claim.point).unwrap();facts.native_started=true;facts.lifecycle=if value.is_some(){StudyPointState::Completed}else{StudyPointState::Failed};facts.scientific=ScientificFacts{usable:value.is_some(),candidate_use:None,seed_permission:false};
    let outcome=PointOutcome{key:OccurrenceKey(key),lifecycle:facts.lifecycle,scientific:facts.scientific.clone(),diagnostic:None,start:None,effect:facts.effect,attempts:vec![]};
    // SAFETY: the immediately sealed fixture attempt backs these original observations.
    unsafe {store.observe_study_point(&claim.point,&facts,&outcome,true).await}.unwrap();
}
fn cohort(physical:&flow::PhysicalSource)->StudyCohortRequest{StudyCohortRequest{filter:StudyWorkbenchFilter{physical_revision:Some(physical.revision.clone()),..Default::default()},maximum_studies:4,maximum_members:8}}
#[tokio::test]
async fn workbench_public_cohort_retains_population_compares_value_variants_and_refuses_incompatible_quantity() {
    use datafusion::arrow::array::{Array,Float64Array,Int64Array};
    let runtime=flow::durable_tests::durable_runtime();
    let physical=runtime.record_physical_source(&flow::tests::physical(),&crate::CancelSource::new()).await.unwrap();
    let store=runtime.canonical_store().unwrap();
    let source=store.edit("workbench-model",None,"first",&[]).await.unwrap();
    create(&runtime,"study-a",&source,&physical,3).await;
    let owner=pse_operations::mint_id();let quantity=pse_operations::mint_id();let unit=pse_operations::mint_id();
    finish(&runtime,"study-a",0,quantity,unit,owner,Some(2.0)).await;
    finish(&runtime,"study-a",1,quantity,unit,owner,None).await;
    let changed=store.edit("workbench-model",Some(&source.key),"value-only",&[]).await.unwrap();
    create(&runtime,"study-b",&changed,&physical,1).await;finish(&runtime,"study-b",0,quantity,unit,owner,Some(8.0)).await;
    let frozen=runtime.freeze_study_cohort(cohort(&physical)).await.unwrap();
    assert_eq!(frozen.document().members.len(),4);assert!(frozen.document().complete);
    assert_eq!(frozen.document().members[1].facts.lifecycle,StudyPointState::Failed);
    assert!(frozen.document().members[2].attempt.is_none());
    assert_ne!(frozen.document().headers[0].source,frozen.document().headers[1].source);
    let kpi=StudyKpiRequest{relation:"runtime.simulation_samples".into(),owner,field:"value".into(),partition:"0".into(),start:0,end:3};
    let result=frozen.aggregate(&kpi,CancellationToken::new()).await.unwrap();
    assert_eq!(result.included,[0,3]);assert_eq!(result.unavailable,[1,2]);
    let batch=result.batches.iter().find(|b|b.num_rows()>0).unwrap();
    assert_eq!(batch.column_by_name("count").unwrap().as_any().downcast_ref::<Int64Array>().unwrap().value(0),6);
    assert_eq!(batch.column_by_name("mean").unwrap().as_any().downcast_ref::<Float64Array>().unwrap().value(0),7.0);
    let mut selected=frozen.result(0,"runtime.simulation_samples",0,1,CancellationToken::new()).await.unwrap();
    assert_eq!(selected.next_batch().await.unwrap().unwrap().num_rows(),1);selected.close();
    assert!(frozen.result(2,"runtime.simulation_samples",0,1,CancellationToken::new()).await.is_err());
    create(&runtime,"study-c",&changed,&physical,1).await;finish(&runtime,"study-c",0,pse_operations::mint_id(),unit,owner,Some(100.0)).await;
    assert_eq!(frozen.document().members.len(),4,"later live membership cannot alter frozen analysis");
    let incompatible=runtime.freeze_study_cohort(cohort(&physical)).await.unwrap();
    assert!(incompatible.aggregate(&kpi,CancellationToken::new()).await.is_err());
    assert!(runtime.freeze_study_cohort(StudyCohortRequest{maximum_members:1,..cohort(&physical)}).await.is_err());
    drop(selected);drop(incompatible);drop(frozen);drop(result);
    store.remove_isolated_fixture().await.unwrap();
}
#[tokio::test]
async fn study_observer_public_reconnect_checks_authority_and_exit_preserves_science() {
    let runtime=flow::durable_tests::durable_runtime();
    let physical=runtime.record_physical_source(&flow::tests::physical(),&crate::CancelSource::new()).await.unwrap();
    let store=runtime.canonical_store().unwrap();let source=store.edit("observer-model",None,"source",&[]).await.unwrap();
    create(&runtime,"observed-study",&source,&physical,1).await;
    let scope=store.study_scope(&point_key("observed-study",OccurrenceKey(0))).await.unwrap();
    let claim=store.claim_study_point(&scope,None,"observer-claim","worker",Duration::from_secs(60)).await.unwrap();
    let request=StudyObservationRequest{study:"observed-study".into(),maximum_members:2,maximum_events:8};
    let mut observer=runtime.observe_study(request.clone(),None).await.unwrap();
    let page=observer.poll().await.unwrap();assert_eq!(page.sources.len(),1);
    assert_eq!(page.sources[0].continuity,StudyObservationContinuity::LiveNumericsUnavailable);
    let cursor=observer.cursor();observer.close();drop(observer);
    assert!(!store.canonical_study("observed-study").await.unwrap().unwrap().cancelled);
    assert!(!store.canonical_run(&claim.point.run).await.unwrap().unwrap().cancelled);
    let reopened=runtime.observe_study(request.clone(),Some(cursor.clone())).await.unwrap();drop(reopened);
    let mut foreign=cursor.clone();foreign.issuing_store.push_str("-foreign");
    assert!(runtime.observe_study(request.clone(),Some(foreign)).await.is_err());
    let mut wrong=cursor;wrong.context.source.push_str("-wrong");
    assert!(runtime.observe_study(request,Some(wrong)).await.is_err());
    // Explicit cancellation remains its own canonical authority after observer exit.
    let cancelled=store.cancel_study("observed-study").await.unwrap();assert!(cancelled.cancelled);
    store.remove_isolated_fixture().await.unwrap();
}
