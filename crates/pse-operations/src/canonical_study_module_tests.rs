// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

// Included only inside the registered canonical-store server test module.
mod surrealism_qualification {
    use super::*;
    use crate::canonical::study_module::StudyModuleReceipt;
    use std::sync::Arc;

    struct Qualification {
        receipt: StudyModuleReceipt,
        pool: usize,
        controllers: usize,
        execution_time: Duration,
    }
    impl Qualification {
        fn selected(store: &CanonicalStore) -> Self {
            let state=store.deployment_state();
            let receipt=StudyModuleReceipt::read_qualification(state).expect("explicit separate qualification receipt required");
            let config:serde_json::Value=serde_json::from_slice(&std::fs::read(state.join("config.json")).unwrap()).unwrap();
            let limits=&config["study_module_qualification"]["limits"];
            let memory=limits["max_memory_bytes"].as_u64().unwrap();
            let server=config["resources"]["server_memory_bytes"].as_u64().unwrap();
            Self {receipt,pool:usize::try_from(limits["max_pool_size"].as_u64().unwrap()).unwrap(),
                controllers:usize::try_from(server/8/memory).unwrap(),
                execution_time:Duration::from_millis(limits["max_execution_time_ms"].as_u64().unwrap())}
        }
        fn module(index:usize)->String {format!("pse_qualification_{index}")}
        async fn install(&self,store:&CanonicalStore,index:usize) {
            // Distinct buckets, rather than module aliases, create distinct native
            // runtime/cache identities for the exact same authenticated bytes.
            let bucket=format!("{}_qualification_{index}",self.receipt.bucket());
            let module=Self::module(index);
            bounded_query(store.db.query(format!("DEFINE BUCKET {bucket} READONLY BACKEND $backend PERMISSIONS NONE; DEFINE MODULE mod::{module} FROM f\"{bucket}:/study.surli\" UNSIGNED PERMISSIONS FULL;"))
                .bind(("backend",self.receipt.backend().unwrap()))).await.unwrap();
            let mut response=bounded_query(store.db.query("LET $info=INFO FOR DB STRUCTURE; LET $module=($info.modules.filter(|$row|$row.name=$name))[0]; RETURN $module.unsigned=true AND $module.executable.type='surrealism' AND $module.executable.bucket=$bucket AND $module.executable.key='study.surli' AND array::sort($module.exports.name)=array::sort(['study::readiness','qualification::trap','qualification::busy','qualification::memory_refused']) AND array::len($module.exports.filter(|$export|$export.writeable))=0;")
                .bind(("name",module)).bind(("bucket",bucket))).await.unwrap();
            assert_eq!(response.take::<Option<bool>>(0).unwrap(),Some(true),"actual installed qualification metadata must match the separately admitted artifact");
        }
        async fn memory_refused(&self,store:&CanonicalStore,index:usize) {
            let mut response=bounded_query(store.db.query(format!("RETURN mod::{}::qualification::memory_refused();",Self::module(index)))).await.unwrap();
            assert_eq!(response.take::<Option<bool>>(0).unwrap(),Some(true),"the actual guest allocator must refuse the 96 MiB allocation under its 64 MiB ceiling");
        }
    }
    fn rollback_query(index:usize,export:&str)->String {
        format!("BEGIN; UPDATE ONLY type::record('canonical_study_readiness',$point) SET eligible=false; RETURN mod::{}::qualification::{export}(); COMMIT;",Qualification::module(index))
    }
    async fn assert_unchanged(store:&CanonicalStore,study:&str,points:usize,cursor:u64) {
        for ordinal in 0..points {
            let point=point_key(study,OccurrenceKey(u32::try_from(ordinal).unwrap()));
            let hint=store.readiness_hint(&point).await.unwrap().unwrap();
            assert!(hint.eligible,"failed/cancelled module call committed its sibling write: {point}");
        }
        assert_eq!(store.readiness_cursor(study).await.unwrap().revision,cursor);
        assert!(store.pending_readiness_page(study).await.unwrap().is_none());
    }
    fn peer(store:&CanonicalStore)->CanonicalStore {
        let mut options=CanonicalOptions::from_state(store.deployment_state()).unwrap();options.database=store.database().to_owned();
        crate::testing::canonical_fixture_peer(store,&options).unwrap()
    }
    #[tokio::test]
    async fn canonical_studies_surrealism_qualification_faults_rollback_and_recover() {
        let (store,_,revision)=fixture().await;
        let study="qualification-faults";
        store.create_study(study,&request(&revision,"qualification-fault-parent"),&[],&[occurrence(&revision,0,vec![])],&||false).await.unwrap();
        let cursor=store.readiness_cursor(study).await.unwrap().revision;
        let qualification=Qualification::selected(&store);qualification.install(&store,0).await;
        qualification.memory_refused(&store,0).await;
        let point=point_key(study,OccurrenceKey(0));
        for export in ["trap","busy"] {
            let started=std::time::Instant::now();
            let result=bounded_query(store.db.query(rollback_query(0,export)).bind(("point",point.clone()))).await;
            assert!(result.is_err(),"{export} must refuse its containing native transaction");
            if export=="busy" {
                assert!(started.elapsed()<qualification.execution_time.saturating_mul(3)+Duration::from_secs(1),"strict synchronous guest interruption must finish within the finite native execution budget plus scheduling tolerance");
            }
            assert_unchanged(&store,study,1,cursor).await;
            qualification.memory_refused(&store,0).await;
        }
        let query=format!("BEGIN; UPDATE ONLY type::record('canonical_study_readiness',$point) SET eligible=false; LET $refused=mod::{}::qualification::memory_refused(); IF $refused {{ THROW 'PSE_REFUSAL:study_readiness_policy_refused'; }}; RETURN $refused; COMMIT;",Qualification::module(0));
        let result=bounded_query(store.db.query(query).bind(("point",point))).await;
        assert!(matches!(result,Err(CanonicalError::Refused{code:pse_schema::catalog::canonical_refusals::RefusalCode::StudyReadinessPolicyRefused,..})),"allocation refusal must roll back the sibling write");
        assert_unchanged(&store,study,1,cursor).await;
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_studies_surrealism_qualification_pool_wait_expiry_and_cancellation_drain() {
        let (store,_,revision)=fixture().await;
        let study="qualification-pool";
        let qualification=Qualification::selected(&store);
        assert_eq!(qualification.pool,4,"this representative qualification deliberately occupies four native pool slots");
        let count=qualification.pool+2;
        let points=(0..count).map(|key|occurrence(&revision,u32::try_from(key).unwrap(),vec![])).collect::<Vec<_>>();
        store.create_study(study,&request(&revision,"qualification-pool-parent"),&[],&points,&||false).await.unwrap();
        let cursor=store.readiness_cursor(study).await.unwrap().revision;
        let peers=(0..count).map(|_|peer(&store)).collect::<Vec<_>>();
        qualification.install(&store,0).await;qualification.memory_refused(&store,0).await;
        let start=Arc::new(tokio::sync::Barrier::new(qualification.pool+1));
        let mut occupied=tokio::task::JoinSet::new();
        for (ordinal,peer) in peers.iter().take(qualification.pool).cloned().enumerate() {
            let start=start.clone();let point=point_key(study,OccurrenceKey(u32::try_from(ordinal).unwrap()));
            occupied.spawn(async move {start.wait().await;bounded_query(peer.db.query(rollback_query(0,"busy")).bind(("point",point))).await});
        }
        start.wait().await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        let expiring=peers[qualification.pool].clone();
        let point=point_key(study,OccurrenceKey(u32::try_from(qualification.pool).unwrap()));
        let started=std::time::Instant::now();
        let expired=within_clock(original_deadline(Duration::from_millis(100)),async {
            bounded_query(expiring.db.query(rollback_query(0,"memory_refused")).bind(("point",point))).await
        }).await;
        assert!(expired.is_err(),"an occupied pool cannot grant the fifth invocation a refreshed clock");
        assert!(started.elapsed()<qualification.execution_time,"the caller's original short clock must expire before the occupying busy calls");
        let cancelled=peers[qualification.pool+1].clone();
        let context=surrealdb::opt::RequestContext::new(std::time::Instant::now()+Duration::from_secs(10))
            .with_lifetime_owner(Arc::new(cancelled.admit_activity().unwrap()));
        let cancellation=context.clone();
        let point=point_key(study,OccurrenceKey(u32::try_from(qualification.pool+1).unwrap()));
        let queued=tokio::spawn(async move {bounded_query(cancelled.db.query(rollback_query(0,"memory_refused")).request_context(context).bind(("point",point))).await});
        tokio::time::sleep(Duration::from_millis(50)).await;cancellation.cancel();
        assert!(queued.await.unwrap().is_err(),"explicit queued cancellation must not become a successful late invocation");
        while let Some(result)=occupied.join_next().await {assert!(result.unwrap().is_err());}
        // Local timeout/abort is not physical terminal proof. Join each actual
        // owned driver before reading rollback state or retiring this database.
        for peer in &peers {peer.disconnect().await.unwrap();}
        assert_unchanged(&store,study,count,cursor).await;
        qualification.memory_refused(&store,0).await;
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_studies_surrealism_qualification_global_capacity_survives_cache_eviction() {
        let (store,_,revision)=fixture().await;
        let study="qualification-global";
        let qualification=Qualification::selected(&store);
        assert_eq!(qualification.pool,4);
        assert!((8..=16).contains(&qualification.controllers),"run this bounded functional control on the explicitly selected 8–16 controller qualification profile");
        let runtime_count=qualification.controllers.div_ceil(qualification.pool);
        let count=qualification.controllers+qualification.pool;
        let points=(0..count).map(|key|occurrence(&revision,u32::try_from(key).unwrap(),vec![])).collect::<Vec<_>>();
        store.create_study(study,&request(&revision,"qualification-global-parent"),&[],&points,&||false).await.unwrap();
        let cursor=store.readiness_cursor(study).await.unwrap().revision;
        let peers=(0..count).map(|_|peer(&store)).collect::<Vec<_>>();
        for index in 0..runtime_count+qualification.pool {qualification.install(&store,index).await;}
        for index in 0..runtime_count {qualification.memory_refused(&store,index).await;}
        let start=Arc::new(tokio::sync::Barrier::new(qualification.controllers+1));
        let mut calls=tokio::task::JoinSet::new();
        for (ordinal,peer) in peers.iter().take(qualification.controllers).cloned().enumerate() {
            let start=start.clone();let index=ordinal/qualification.pool;
            let point=point_key(study,OccurrenceKey(u32::try_from(ordinal).unwrap()));
            calls.spawn(async move {start.wait().await;bounded_query(peer.db.query(rollback_query(index,"busy")).bind(("point",point))).await});
        }
        start.wait().await;tokio::time::sleep(Duration::from_millis(50)).await;
        // New runtime keys churn the bounded cache while previous invocations
        // still own their physical Stores. Eviction cannot return those permits.
        for (offset,peer) in peers.iter().skip(qualification.controllers).cloned().enumerate() {
            let index=runtime_count+offset;let ordinal=qualification.controllers+offset;
            let point=point_key(study,OccurrenceKey(u32::try_from(ordinal).unwrap()));
            calls.spawn(async move {bounded_query(peer.db.query(rollback_query(index,"busy")).bind(("point",point))).await});
        }
        let mut capacity_refusals=0;
        while let Some(result)=calls.join_next().await {
            let result=result.unwrap();
            if matches!(&result,Err(CanonicalError::Refused{code:pse_schema::catalog::canonical_refusals::RefusalCode::SurrealismControllerCapacityExceeded,..})) {capacity_refusals+=1;}
            assert!(result.is_err(),"busy/capacity calls cannot commit their sibling writes");
        }
        for peer in &peers {peer.disconnect().await.unwrap();}
        assert_unchanged(&store,study,count,cursor).await;
        // A typed real native refusal is required. Error prose, timeout counts
        // and launch arithmetic do not establish process-wide saturation.
        assert!(capacity_refusals>0,"no typed process-wide capacity refusal was observed; global saturation remains unqualified");
        for index in 0..runtime_count+qualification.pool {qualification.memory_refused(&store,index).await;}
        store.remove_isolated_fixture().await.unwrap();
    }
}
