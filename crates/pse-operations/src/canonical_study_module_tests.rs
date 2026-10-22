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
            let mut response=bounded_query(store.db.query("LET $info=INFO FOR DB STRUCTURE; LET $module=($info.modules.filter(|$row|$row.name=$name))[0]; RETURN $module.unsigned=true AND $module.executable.type='surrealism' AND $module.executable.bucket=$bucket AND $module.executable.key='/study.surli' AND array::sort($module.exports.name)=array::sort(['study::readiness','qualification::trap','qualification::busy','qualification::memory_refused','qualification::hold']) AND array::len($module.exports.filter(|$export|$export.writeable))=0;")
                .bind(("name",module)).bind(("bucket",bucket))).await.unwrap();
            let index=response.num_statements().saturating_sub(1);
            assert_eq!(response.take::<Option<bool>>(index).unwrap(),Some(true),"actual installed qualification metadata must match the separately admitted artifact");
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
            assert!(hint.eligible,"failed module call committed its sibling write at ordinal {ordinal}: {point}");
        }
        assert_eq!(store.readiness_cursor(study).await.unwrap().revision,cursor);
        assert!(store.pending_readiness_page(study).await.unwrap().is_none());
    }
    fn peer(store:&CanonicalStore)->CanonicalStore {
        let mut options=CanonicalOptions::from_state(store.deployment_state()).unwrap();options.database=store.database().to_owned();
        crate::testing::canonical_fixture_peer(store,&options).unwrap()
    }
    #[derive(Clone,Copy,Debug)]
    struct HoldInterval {start:u64,end:u64}
    impl HoldInterval {
        fn decoded(mut response:surrealdb::IndexedResults)->Self {
            let mut interval=None;
            for index in 0..response.num_statements() {
                match response.take::<Value>(index).unwrap() {
                    Value::None|Value::Null=>{},
                    Value::Array(values)=>{
                        assert!(interval.is_none() && values.len()==2,"one exact hold interval required");
                        let start=codec::decode_uint(values[0].clone()).unwrap();
                        let end=codec::decode_uint(values[1].clone()).unwrap();
                        assert!(end>=start+499_000,"the successful guest must actually hold its controller for the selected 500 ms");
                        interval=Some(Self{start,end});
                    },
                    other=>panic!("unexpected hold result carrier {other:?}"),
                }
            }
            interval.expect("the successful native hold must return its actual guest interval")
        }
        fn covers(self,start:u64,end:u64)->bool {self.start<=start && self.end>=end}
    }
    fn utc_micros()->u64 {
        u64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros()).unwrap()
    }
    fn hold_query(index:usize)->String {
        format!("BEGIN; UPDATE ONLY type::record('canonical_study_readiness',$point) SET eligible=false RETURN NONE; RETURN mod::{}::qualification::hold(); COMMIT;",Qualification::module(index))
    }
    async fn assert_canaries(store:&CanonicalStore,study:&str,committed:&[bool],cursor:u64) {
        for (ordinal,committed) in committed.iter().enumerate() {
            let hint=store.readiness_hint(&point_key(study,OccurrenceKey(u32::try_from(ordinal).unwrap()))).await.unwrap().unwrap();
            assert_eq!(!hint.eligible,*committed,"native canary at ordinal {ordinal} must match the actual acknowledged transaction outcome");
        }
        assert_eq!(store.readiness_cursor(study).await.unwrap().revision,cursor);
        assert!(store.pending_readiness_page(study).await.unwrap().is_none());
    }
    async fn warm_pool(peers:&[CanonicalStore],pool:usize) {
        let mut calls=tokio::task::JoinSet::new();
        let start=Arc::new(tokio::sync::Barrier::new(peers.len()+1));
        for (ordinal,peer) in peers.iter().cloned().enumerate() {
            let start=start.clone();let index=ordinal/pool;
            calls.spawn(async move {
                start.wait().await;
                HoldInterval::decoded(bounded_query(peer.db.query(format!("RETURN mod::{}::qualification::hold();",Qualification::module(index)))).await.unwrap())
            });
        }
        start.wait().await;
        while let Some(result)=calls.join_next().await {result.unwrap();}
    }
    fn peak_overlap(intervals:&[HoldInterval])->usize {
        let mut edges=intervals.iter().flat_map(|interval|[(interval.start,1_i32),(interval.end,-1)]).collect::<Vec<_>>();
        edges.sort_unstable();
        let mut active=0_i32;let mut peak=0_i32;
        for (_,change) in edges {active+=change;peak=peak.max(active);}
        usize::try_from(peak).unwrap()
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
    async fn canonical_studies_surrealism_qualification_pool_original_expiry_cannot_start_late() {
        let (store,_,revision)=fixture().await;
        let study="qualification-pool";
        let qualification=Qualification::selected(&store);
        assert_eq!(qualification.pool,4);
        let count=qualification.pool+1;
        let points=(0..count).map(|key|occurrence(&revision,u32::try_from(key).unwrap(),vec![])).collect::<Vec<_>>();
        store.create_study(study,&request(&revision,"qualification-pool-parent"),&[],&points,&||false).await.unwrap();
        let cursor=store.readiness_cursor(study).await.unwrap().revision;
        let peers=(0..count).map(|_|peer(&store)).collect::<Vec<_>>();
        qualification.install(&store,0).await;
        warm_pool(&peers[..qualification.pool],qualification.pool).await;
        let start=Arc::new(tokio::sync::Barrier::new(qualification.pool+1));
        let mut occupied=tokio::task::JoinSet::new();
        for (ordinal,peer) in peers.iter().take(qualification.pool).cloned().enumerate() {
            let start=start.clone();let point=point_key(study,OccurrenceKey(u32::try_from(ordinal).unwrap()));
            occupied.spawn(async move {start.wait().await;HoldInterval::decoded(bounded_query(peer.db.query(hold_query(0)).bind(("point",point))).await.unwrap())});
        }
        start.wait().await;tokio::time::sleep(Duration::from_millis(50)).await;
        let expiring=peers[qualification.pool].clone();
        let point=point_key(study,OccurrenceKey(u32::try_from(qualification.pool).unwrap()));
        let submitted=utc_micros();let original_expiry=submitted+100_000;
        let expired=within_clock(original_deadline(Duration::from_millis(100)),async {
            bounded_query(expiring.db.query(rollback_query(0,"memory_refused")).bind(("point",point))).await
        }).await;
        let mut intervals=Vec::new();
        while let Some(result)=occupied.join_next().await {intervals.push(result.unwrap());}
        for peer in &peers {peer.disconnect().await.unwrap();}
        assert_eq!(intervals.len(),qualification.pool);
        assert!(intervals.iter().all(|interval|interval.covers(submitted,original_expiry)),"actual successful guest intervals must certify all four slots occupied through the fifth call's original expiry: {intervals:?}");
        assert!(expired.is_err(),"the occupied pool cannot give an expired call a fresh budget");
        let mut committed=vec![true;qualification.pool];committed.push(false);
        assert_canaries(&store,study,&committed,cursor).await;
        qualification.memory_refused(&store,0).await;
        store.remove_isolated_fixture().await.unwrap();
    }
    async fn settlement(store:&CanonicalStore,operation:&str,study:&str,run:&str)->Option<Object> {
        let mut response=bounded_query(store.db.query("RETURN fn::pse_study_v1::readiness_settlement($pse_rpc_expires_at,$operation,$study,$run);")
            .bind(("operation",operation.to_owned())).bind(("study",study.to_owned())).bind(("run",run.to_owned()))).await.unwrap();
        response.take::<Option<Object>>(0).unwrap()
    }
    #[tokio::test]
    async fn canonical_studies_surrealism_qualification_cancelled_apply_settles_original_receipt() {
        let (store,_,revision)=fixture().await;
        let qualification=Qualification::selected(&store);qualification.install(&store,0).await;
        // Wrap the actual generated policy body only in this UUID fixture. The
        // same admitted production module still makes every scientific decision.
        let start=wire::SCHEMA.find("DEFINE FUNCTION fn::pse_study_v1::readiness_policy(").unwrap();
        let tail=&wire::SCHEMA[start..];let end=tail.find("\n};").unwrap()+4;
        let original=&tail[..end];
        let delayed=original.replacen("DEFINE FUNCTION ","DEFINE FUNCTION OVERWRITE ",1)
            .replacen("    RETURN mod::pse::study::readiness",&format!("    mod::{}::qualification::hold();\n    RETURN mod::pse::study::readiness",Qualification::module(0)),1);
        bounded_query(store.db.query(delayed)).await.unwrap();
        for committed_before_recovery in [false,true] {
            let study=if committed_before_recovery {"cancel-after-commit"} else {"cancel-before-fence"};
            let run=format!("{study}-parent");
            store.create_study(study,&request(&revision,&run),&[],&[occurrence(&revision,0,vec![])],&||false).await.unwrap();
            let cursor=store.readiness_cursor(study).await.unwrap();
            let coordinates=readiness_coordinates(&store,study,&cursor,"verify",None).unwrap();
            let operation=format!("qualification-cancel:{}",uuid::Uuid::new_v4());
            let pending=prepare_readiness_test(&store,&coordinates,&operation).await;
            let keys=pending.keys.clone();
            let original_epoch=pending.epoch.clone();
            let caller=peer(&store);
            let context=surrealdb::opt::RequestContext::new(std::time::Instant::now()+Duration::from_secs(10))
                .with_lifetime_owner(Arc::new(caller.admit_activity().unwrap()));
            let cancellation=context.clone();let dispatched=context.clone();
            let body_operation=operation.clone();let body_run=run.clone();let body_epoch=original_epoch.clone();let body_caller=caller.clone();
            let applying=tokio::spawn(async move {
                bounded_query(body_caller.db.query("RETURN fn::pse_study_v1::readiness_apply_page($pse_rpc_expires_at,$operation,$study,$run,$epoch,65);")
                    .bind(("operation",body_operation)).bind(("study",study)).bind(("run",body_run)).bind(("epoch",body_epoch)).request_context(context)).await
            });
            within_clock(original_deadline(Duration::from_secs(5)),async {
                while !dispatched.was_dispatched() {tokio::time::sleep(Duration::from_millis(1)).await;}
                Ok(())
            }).await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;cancellation.cancel();
            assert!(applying.await.unwrap().is_err(),"the cancelled caller must not invent success");
            if committed_before_recovery {
                within_clock(original_deadline(Duration::from_secs(5)),async {
                    while settlement(&store,&operation,study,&run).await.is_none() {tokio::time::sleep(Duration::from_millis(10)).await;}
                    Ok(())
                }).await.unwrap();
            }
            // The original exact owner survives local cancellation. Recovery
            // either reads its immutable commit or fences that same epoch before
            // applying the original page, never a freshly allocated operation.
            assert_eq!(store.resume_readiness_page(pending,true).await.unwrap(),1);
            caller.disconnect().await.unwrap();
            let receipt=settlement(&store,&operation,study,&run).await.unwrap();
            validate_repair_outcomes(&receipt,&keys).unwrap();
            assert_eq!(store.readiness_cursor(study).await.unwrap().revision,cursor.revision+1);
            assert!(store.pending_readiness_page(study).await.unwrap().is_none());
            let mut response=bounded_query(store.db.query("SELECT * FROM canonical_execution_operations WHERE study=$study AND kind='study-readiness-repair-page';").bind(("study",study))).await.unwrap();
            let rows=response.take::<Vec<Object>>(0).unwrap();assert_eq!(rows.len(),1);
            assert_eq!(rows[0].get("key"),Some(&Value::String(operation.clone())));
            assert_eq!(rows[0].get("disposition"),Some(&Value::String("settled".into())));
            assert_eq!(settlement(&store,&operation,study,&run).await.unwrap(),receipt);
            assert_eq!(store.readiness_cursor(study).await.unwrap().revision,cursor.revision+1);
        }
        bounded_query(store.db.query(original.replacen("DEFINE FUNCTION ","DEFINE FUNCTION OVERWRITE ",1))).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_studies_surrealism_qualification_global_capacity_survives_cache_eviction() {
        let (store,_,revision)=fixture().await;
        let study="qualification-global";let qualification=Qualification::selected(&store);
        assert_eq!(qualification.pool,4);
        assert!((8..=16).contains(&qualification.controllers));
        let runtime_count=qualification.controllers.div_ceil(qualification.pool);
        let count=qualification.controllers+qualification.pool;
        let points=(0..count).map(|key|occurrence(&revision,u32::try_from(key).unwrap(),vec![])).collect::<Vec<_>>();
        store.create_study(study,&request(&revision,"qualification-global-parent"),&[],&points,&||false).await.unwrap();
        let cursor=store.readiness_cursor(study).await.unwrap().revision;
        let peers=(0..count).map(|_|peer(&store)).collect::<Vec<_>>();
        for index in 0..runtime_count+qualification.pool {qualification.install(&store,index).await;}
        warm_pool(&peers[..qualification.controllers],qualification.pool).await;
        let start=Arc::new(tokio::sync::Barrier::new(qualification.controllers+1));
        let mut calls=tokio::task::JoinSet::new();
        for (ordinal,peer) in peers.iter().take(qualification.controllers).cloned().enumerate() {
            let start=start.clone();let index=ordinal/qualification.pool;
            let point=point_key(study,OccurrenceKey(u32::try_from(ordinal).unwrap()));
            calls.spawn(async move {start.wait().await;(ordinal,bounded_query(peer.db.query(hold_query(index)).bind(("point",point))).await)});
        }
        start.wait().await;tokio::time::sleep(Duration::from_millis(50)).await;
        let churn_started=utc_micros();
        for (offset,peer) in peers.iter().skip(qualification.controllers).cloned().enumerate() {
            let index=runtime_count+offset;let ordinal=qualification.controllers+offset;
            let point=point_key(study,OccurrenceKey(u32::try_from(ordinal).unwrap()));
            calls.spawn(async move {(ordinal,bounded_query(peer.db.query(hold_query(index)).bind(("point",point))).await)});
        }
        let mut capacity_refusals=0;let mut intervals=Vec::new();let mut committed=vec![false;count];
        while let Some(result)=calls.join_next().await {
            let (ordinal,result)=result.unwrap();
            match result {
                Ok(response)=>{committed[ordinal]=true;intervals.push((ordinal,HoldInterval::decoded(response)));},
                Err(CanonicalError::Refused{code:pse_schema::catalog::canonical_refusals::RefusalCode::SurrealismControllerCapacityExceeded,..})=>capacity_refusals+=1,
                Err(error)=>panic!("unexpected native controller outcome for {ordinal}: {error}"),
            }
        }
        for peer in &peers {peer.disconnect().await.unwrap();}
        assert_canaries(&store,study,&committed,cursor).await;
        let initial=intervals.iter().filter(|(ordinal,_)|*ordinal<qualification.controllers).map(|(_,interval)|*interval).collect::<Vec<_>>();
        assert_eq!(initial.len(),qualification.controllers,"all prewarmed initial holders must acquire their actual controller");
        assert!(initial.iter().all(|interval|interval.covers(churn_started,churn_started)),"guest intervals must certify physical controller ownership while new cache keys enter: {initial:?}");
        assert_eq!(peak_overlap(&initial),qualification.controllers,"actual guest intervals, not launch arithmetic, must establish cooperative overlap beyond the four host workers");
        assert!(capacity_refusals>0,"no typed process-wide capacity refusal observed; saturation remains unqualified");
        for index in 0..runtime_count+qualification.pool {qualification.memory_refused(&store,index).await;}
        store.remove_isolated_fixture().await.unwrap();
    }
}
