// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Included in the canonical study server test module; uses its isolated fixture.

#[tokio::test]
async fn capture_locator_profile_admits_native_wrappers_and_refuses_deeper_values() {
    use crate::canonical::MetadataResponse;
    let (store,_,_)=fixture().await;
    // The exact locator nesting adds source/page wrappers around native rows.
    // A manifest can occupy nearly the whole native byte-bound page, while a
    // native record id adds RecordId/RecordIdKey protobuf message levels.
    let shape="RETURN {items:[{position:0dec,manifest:{descriptors:$descriptors},page:{blocks:[{id:type::record('canonical_result_blocks','probe')}],batches:[],indexes:[],next:NONE,exhausted:true}}],member:1dec,after:NONE,exhausted:true};";
    let descriptors=Bytes::from(vec![0;pse_schema::catalog::study_workbench::STUDY_PAGE_BYTES-4096]);
    let mut response=bounded_query(store.db.metadata_query(MetadataResponse::CaptureLocators,shape).bind(("descriptors",descriptors))).await.unwrap();
    assert!(matches!(response.take::<Value>(0).unwrap(),Value::Object(_)));
    drop(response);
    let shallow=bounded_query(store.db.metadata_query(MetadataResponse::StudyPage,shape).bind(("descriptors",Bytes::from(vec![])))).await.unwrap_err();
    assert!(shallow.to_string().contains("depth envelope"),"ordinary study-page depth must remain closed: {shallow:?}");
    let deeper=shape.replace("id:type::record('canonical_result_blocks','probe')","extra:{extra:{extra:{extra:true}}}");
    let error=bounded_query(store.db.metadata_query(MetadataResponse::CaptureLocators,deeper).bind(("descriptors",Bytes::from(vec![])))).await.unwrap_err();
    assert!(error.to_string().contains("depth envelope"),"locator endpoint cannot admit arbitrary deeper values: {error:?}");
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn capture_last_owner_drain_refuses_exact_replay_until_native_release() {
    use crate::canonical_study_workbench::{StudyWorkbenchFilter,pause_capture_release};
    let (store,_,revision)=fixture().await;
    let points=(0..3).map(|n|occurrence(&revision,n,vec![])).collect::<Vec<_>>();
    store.create_study("capture-drain-study",&request(&revision,"capture-drain-parent"),&[],&points,&||false).await.unwrap();
    let filter=StudyWorkbenchFilter{study:Some("capture-drain-study".into()),..Default::default()};
    let key=format!("capture-drain-{}",uuid::Uuid::new_v4());
    let selected=store.capture_study_workbench(&key,&filter,1,3).await.unwrap();
    let replay=store.capture_study_workbench(&key,&filter,1,3).await.unwrap();
    assert_eq!(selected.capture.key(),replay.capture.key());
    let pause=pause_capture_release(&key);
    drop(selected);
    assert_eq!(replay.capture.member_page(None,3).await.unwrap().rows.len(),3,"one replay handle cannot release another handle's facts");
    drop(replay);
    tokio::time::timeout(Duration::from_secs(5),pause.entered.notified()).await.unwrap();
    assert!(matches!(store.capture_study_workbench(&key,&filter,1,3).await,Err(CanonicalError::CaptureDraining{..})));
    let mut facts=bounded_query(store.db.query("SELECT key FROM canonical_study_capture_members WHERE capture=$capture;").bind(("capture",key.clone()))).await.unwrap();
    assert_eq!(facts.take::<Vec<Object>>(0).unwrap().len(),3,"paused cleanup retains all frozen facts");
    pause.resume.notify_one();
    store.result_read_drain.drain().await.unwrap();
    let mut closed=bounded_query(store.db.query("SELECT state FROM ONLY type::record('canonical_study_captures',$capture); SELECT key FROM canonical_study_capture_members WHERE capture=$capture;").bind(("capture",key))).await.unwrap();
    assert_eq!(closed.take::<Option<Object>>(0).unwrap().unwrap().get("state"),Some(&Value::String("released".into())));
    assert!(closed.take::<Vec<Object>>(1).unwrap().is_empty());
}

#[tokio::test]
async fn capture_expired_gate_never_bypasses_unchanged_incarnation() {
    let (store,_,_)=fixture().await;
    let owner=store.registered_file_owner().await.unwrap();
    let key=format!("capture-expiry-{}",uuid::Uuid::new_v4());
    let request=Bytes::from(vec![1,2,3]);
    let mut opened=protected_query("capture-test-open",||Ok(store.db.query("RETURN fn::pse_workbench_v1::open_capture(time::micros()+100000,$capture,$owner,$request);").bind(("capture",key.clone())).bind(("owner",owner.clone())).bind(("request",request.clone())))).await.unwrap();
    let opened=opened.take::<Option<Object>>(0).unwrap().unwrap();
    tokio::time::sleep(Duration::from_millis(110)).await;
    let busy=bounded_query(store.db.query("RETURN fn::pse_workbench_v1::publication($pse_rpc_expires_at);")).await.unwrap_err();
    assert!(matches!(busy,CanonicalError::Refused{code:crate::canonical::RefusalCode::StudyCapturePublicationBusy,..}));
    let mut recovery=protected_query("capture-test-recover",||Ok(store.db.query("RETURN fn::pse_workbench_v1::recover_capture($pse_rpc_expires_at,$capture,$owner,$request); SELECT incarnation FROM ONLY canonical_study_capture_gate:current; RETURN fn::pse_workbench_v1::publication($pse_rpc_expires_at);").bind(("capture",key.clone())).bind(("owner",owner.clone())).bind(("request",request.clone())))).await.unwrap();
    let new=recovery.take::<Option<Object>>(1).unwrap().unwrap();
    assert_ne!(opened.get("incarnation"),new.get("incarnation"),"explicit recovery mutates the guard to fence an in-flight stale capture");
    assert_eq!(recovery.take::<Option<bool>>(2).unwrap(),Some(true));
    let late=bounded_query(store.db.query("RETURN fn::pse_workbench_v1::commit_capture($pse_rpc_expires_at,$capture,$owner,$request,[],1,1);").bind(("capture",key.clone())).bind(("owner",owner.clone())).bind(("request",request.clone()))).await.unwrap_err();
    assert!(matches!(late,CanonicalError::Refused{code:crate::canonical::RefusalCode::StudyCaptureOriginalGrantClosed,..}));
    crate::canonical_study_workbench::release_capture_custody(&store,&key,&owner,request.as_ref()).await.unwrap();
}

#[tokio::test]
#[allow(clippy::print_stderr,reason="native clock control distinguishes request latency from fixture settlement")]
async fn capture_busy_wait_preserves_computed_append_and_original_clock() {
    use crate::canonical_execution::ResultPayload;
    let started=std::time::Instant::now();
    let (store,_,revision)=fixture().await;
    let run=request(&revision,"capture-busy-run");
    store.begin_run(&run).await.unwrap();
    let fence=store.claim_run(run.key.as_str(),"capture-busy-claim","worker",Duration::from_secs(60)).await.unwrap();
    let owner=store.registered_file_owner().await.unwrap();
    let key=format!("capture-busy-{}",uuid::Uuid::new_v4());
    let intent=Bytes::from(vec![5]);
    protected_query("capture-test-gate",||Ok(store.db.query("RETURN fn::pse_workbench_v1::open_capture($pse_rpc_expires_at,$capture,$owner,$request);").bind(("capture",key.clone())).bind(("owner",owner.clone())).bind(("request",intent.clone())))).await.unwrap();
    // The computed output moves once into the publication future. Neither wait
    // path has a callback capable of recomputing it or obtaining a fresh grant.
    let computed=ResultPayload::new(vec![7,8,9]).unwrap();
    let setup=started.elapsed();
    let deadline=original_deadline(Duration::from_secs(3));
    let writer=store.clone(); let selected=fence.clone();
    let pending=tokio::spawn(async move {
        within_clock(deadline,writer.append_result_batch(&selected,"capture-busy-append","values",0,computed,1)).await
    });
    let reader=store.clone();
    let native_wait=tokio::spawn(async move {
        within_clock(deadline,protected_query("capture-test-global-busy",||Ok(reader.db.query("RETURN fn::pse_workbench_v1::publication($pse_rpc_expires_at);")))).await
    });
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert!(!pending.is_finished(),"bounded availability waiting must outlive four short backoffs");
    assert!(!native_wait.is_finished());
    let expiry_started=std::time::Instant::now();
    let expired=within_clock(original_deadline(Duration::from_millis(100)),store.append_result_batch(&fence,"capture-busy-expired","values",1,ResultPayload::new(vec![10]).unwrap(),1)).await;
    let expiry_elapsed=expiry_started.elapsed();
    // The raw gate is deliberately held by this test. Settle it and join every
    // admitted child before assertions: a panic while it remains pending would
    // block file-owner drain and add a separate 30-second cleanup timeout.
    let recovered=protected_query("capture-test-open-publication",||Ok(store.db.query("RETURN fn::pse_workbench_v1::recover_capture($pse_rpc_expires_at,$capture,$owner,$request);").bind(("capture",key.clone())).bind(("owner",owner.clone())).bind(("request",intent.clone())))).await;
    let (pending,native_wait)=tokio::join!(pending,native_wait);
    let released=crate::canonical_study_workbench::release_capture_custody(&store,&key,&owner,intent.as_ref()).await;
    let rows=bounded_query(store.db.query("SELECT key FROM canonical_execution_operations WHERE key IN ['capture-busy-append','capture-busy-expired']; SELECT key FROM canonical_result_batches WHERE attempt=$attempt;").bind(("attempt",fence.attempt().to_owned()))).await;
    assert!(recovered.is_ok(),"original gate recovery: {recovered:?}");
    assert!(released.is_ok(),"original gate release: {released:?}");
    let original_timeout=match &expired {
        Err(CanonicalError::Timeout)=>true,
        Err(CanonicalError::SettlementUnresolved{owner,operation,request,source,readback:Some(readback)})=>{
            *owner=="execution-append"&&operation=="capture-busy-expired"&&!request.is_empty()
                &&matches!(source.as_ref(),CanonicalError::Timeout)
                &&match readback.as_ref() {
                    CanonicalError::Timeout=>true,
                    CanonicalError::Driver(error)=>matches!(error.connection_details(),Some(surrealdb_types::ConnectionError::ConnectionFailed))
                        &&error.message()=="Request interrupted: not dispatched",
                    _=>false,
                }
        }
        _=>false,
    };
    assert!(original_timeout,"100ms original append returned {expired:?}; append_elapsed={expiry_elapsed:?}, setup={setup:?}, total_before_assert={:?}",started.elapsed());
    assert!(expired.as_ref().unwrap_err().effect_is_unknown(),"a failed deadline-bound readback never becomes a no-effect proof");
    assert!(expiry_elapsed<Duration::from_secs(1),"100ms append exhausted its original clock: {expiry_elapsed:?}");
    assert_eq!(pending.unwrap().unwrap().row_count,1);
    assert!(native_wait.unwrap().is_ok());
    let mut rows=rows.unwrap();
    let operations=rows.take::<Vec<Object>>(0).unwrap();
    assert_eq!(operations.len(),1);
    assert_eq!(operations[0].get("key"),Some(&Value::String("capture-busy-append".into())),"the expired operation has no committed effect after original gate settlement");
    let batches=rows.take::<Vec<Object>>(1).unwrap();
    assert_eq!(batches.len(),1,"expired publication never resumes with new authority");
    let set=crate::canonical_execution::result_set_key(fence.attempt(),"values");
    assert_eq!(batches[0].get("key"),Some(&Value::String(crate::canonical_execution::result_batch_key(fence.attempt(),&set,0))));
    eprintln!("capture busy clock phases: setup={setup:?}, expired_append={expiry_elapsed:?}, settled_total={:?}, expired_outcome={}",started.elapsed(),if matches!(expired,Err(CanonicalError::Timeout)){"timeout"}else{"unresolved original timeout with failed readback"});
}

#[tokio::test]
async fn installation_expired_issued_page_replays_its_original_grant() {
    use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
    let (store,_,revision)=fixture().await;
    let points=(0..3).map(|key|occurrence(&revision,key,if key==2 {vec![]}else{vec![Dependency::Ordering(OccurrenceKey(key+1))]})).collect::<Vec<_>>();
    let cancelled=Arc::new(AtomicBool::new(false));let signal=cancelled.clone();
    let parent=request(&revision,"expired-page-parent");
    let created=CREATION_OBSERVER.scope(Box::new(move|boundary|{if boundary==CreationBoundary::NodesInstalled {signal.store(true,Ordering::SeqCst);}}),store.create_study("expired-page",&parent,&[],&points,&||cancelled.load(Ordering::SeqCst))).await.unwrap();
    assert!(created.is_none());
    let mut state=bounded_query(store.db.query("SELECT digest FROM ONLY type::record('canonical_study_installations','expired-page');")).await.unwrap();
    let mut state=state.take::<Option<Object>>(0).unwrap().unwrap();
    let digest=codec::decode_string(codec::required(&mut state,"digest").unwrap()).unwrap();
    let original=Bytes::from(vec![11_u8]);
    let mut issue=protected_query("installation-test-issue",||Ok(store.db.query("RETURN fn::pse_study_v1::issue_page(time::micros()+100000,'expired-page',$digest,'edges',0dec,$request,1);").bind(("digest",digest.clone())).bind(("request",original.clone())))).await.unwrap();
    let issued=issue.take::<Option<Object>>(0).unwrap().unwrap();
    tokio::time::sleep(Duration::from_millis(110)).await;
    let mut replay=protected_query("installation-test-replay",||Ok(store.db.query("RETURN fn::pse_study_v1::issue_page($pse_rpc_expires_at,'expired-page',$digest,'edges',0dec,$request,1);").bind(("digest",digest.clone())).bind(("request",original.clone())))).await.unwrap();
    let replay=replay.take::<Option<Object>>(0).unwrap().unwrap();
    assert_eq!(issued.get("issued_expires_at"),replay.get("issued_expires_at"));
    let expired=bounded_query(store.db.query("RETURN fn::pse_study_v1::install_page($pse_rpc_expires_at,'expired-page',$digest,'edges',0dec,$request,[],[],[]);").bind(("digest",digest)).bind(("request",original))).await.unwrap_err();
    assert!(matches!(expired,CanonicalError::Refused{code:crate::canonical::RefusalCode::StudyInstallationOriginalPageExpired,..}));
    let header=store.canonical_study("expired-page").await.unwrap().unwrap();
    assert!(!header.active);
    assert_eq!(header.installed_dependencies,0);
}

#[tokio::test]
async fn capture_all_unavailable_members_retain_the_named_parent_summary() {
    use crate::canonical_study_workbench::StudyWorkbenchFilter;
    let (store,_,revision)=fixture().await;
    store.create_study("capture-parent",&request(&revision,"capture-parent-run"),&[],&[occurrence(&revision,0,vec![])],&||false).await.unwrap();
    store.cancel_study("capture-parent").await.unwrap();
    let scope=store.study_scope(&point_key("capture-parent",OccurrenceKey(0))).await.unwrap();
    store.refuse_study_point(&scope,&scope.action(None).unwrap(),None).await.unwrap();
    let summary=store.canonical_study_summary("capture-parent").await.unwrap().unwrap();
    let fence=store.begin_study_finalization(&summary,"summary",Duration::from_secs(60)).await.unwrap().unwrap();
    let closed=store.close_result_ingestion(&fence,"capture-parent-close").await.unwrap();
    let manifest=store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: this fixture owns the exact effect-free cancelled point and parent summary.
    unsafe {store.seal_study_summary("capture-parent",&manifest,"capture-parent-seal",TerminalClass::Cancelled,b"cancelled fixture").await}.unwrap();
    let selected=store.capture_study_workbench("capture-parent-cut",&StudyWorkbenchFilter{study:Some("capture-parent".into()),..Default::default()},1,1).await.unwrap();
    assert!(selected.capture.member_page(None,1).await.unwrap().rows.iter().all(|member|member.value.manifest.is_none()));
    assert_eq!(selected.capture.header_page(None,1).await.unwrap().rows[0].value.result_attempt.as_deref(),Some(fence.attempt()));
    let mut protected=bounded_query(store.db.query("SELECT run,attempt,manifest FROM canonical_study_capture_protections WHERE capture='capture-parent-cut';")).await.unwrap();
    let protected=protected.take::<Vec<Object>>(0).unwrap();
    assert_eq!(protected.len(),1,"the named parent is protected without counting it as a point result");
    assert_eq!(protected[0].get("run"),Some(&Value::String(fence.run().to_owned())));
    store.forget_study_results("capture-parent").await.unwrap();
    assert!(matches!(store.forget_run_results(fence.run()).await,Err(CanonicalError::Refused{code:crate::canonical::RefusalCode::RunResultsHaveProtectedReaders,..})));
    drop(selected);
    store.result_read_drain.drain().await.unwrap();
    store.forget_run_results(fence.run()).await.unwrap();
}

#[tokio::test]
async fn capture_opaque_public_ids_and_protection_tuple_keys_do_not_alias() {
    use crate::canonical_study_workbench::StudyWorkbenchFilter;
    let (store,_,revision)=fixture().await;
    store.create_study("opaque-study",&request(&revision,"opaque-parent"),&[],&[occurrence(&revision,0,vec![])],&||false).await.unwrap();
    let scope=store.study_scope(&point_key("opaque-study",OccurrenceKey(0))).await.unwrap();
    let claim=store.claim_study_point(&scope,None,"opaque-claim","worker",Duration::from_secs(60)).await.unwrap();
    fail(&store,&claim).await;
    let filter=StudyWorkbenchFilter{study:Some("opaque-study".into()),..Default::default()};
    let first=store.capture_study_workbench("opaque:prefix",&filter,1,1).await.unwrap();
    let second=store.capture_study_workbench("opaque",&filter,1,1).await.unwrap();
    // These physical tuples collided under raw delimiter concatenation. All
    // inputs are admitted opaque strings; a UUID-only restriction is unnecessary.
    let mut keys=bounded_query(store.db.query("RETURN [fn::pse_workbench_v1::protection_key('opaque:prefix','study',$attempt),fn::pse_workbench_v1::protection_key('opaque','prefix:study',$attempt)];").bind(("attempt",claim.fence.attempt().to_owned()))).await.unwrap();
    let keys=keys.take::<Vec<String>>(0).unwrap();
    assert_ne!(keys[0],keys[1]);
    drop(first);
    store.result_read_drain.join_pending_releases().await.unwrap();
    let mut references=bounded_query(store.db.query("SELECT capture FROM canonical_study_capture_protections WHERE capture IN ['opaque:prefix','opaque'];")).await.unwrap();
    let references=references.take::<Vec<Object>>(0).unwrap();
    assert_eq!(references.len(),1);
    assert_eq!(references[0].get("capture"),Some(&Value::String("opaque".into())));
    assert_eq!(second.capture.member_page(None,1).await.unwrap().rows.len(),1);
    drop(second);
    store.result_read_drain.drain().await.unwrap();
}

#[tokio::test]
async fn capture_controlled_lost_cleanup_ack_recovers_the_original_release() {
    use crate::canonical_study_workbench::{StudyWorkbenchFilter,lose_capture_release_reply};
    let (store,_,revision)=fixture().await;
    store.create_study("release-loss-study",&request(&revision,"release-loss-parent"),&[],&[occurrence(&revision,0,vec![])],&||false).await.unwrap();
    let before=store.canonical_study("release-loss-study").await.unwrap().unwrap();
    let filter=StudyWorkbenchFilter{study:Some("release-loss-study".into()),..Default::default()};
    let selected=store.capture_study_workbench("release-loss-cut",&filter,1,1).await.unwrap();
    lose_capture_release_reply("release-loss-cut");
    drop(selected);
    let failed=store.result_read_drain.drain().await.unwrap_err();
    assert!(failed.effect_is_unknown());
    assert!(matches!(failed,CanonicalError::ProtectionRelease{ref source} if matches!(source.as_ref(),CanonicalError::IncompleteResponse)));
    assert!(matches!(store.capture_study_workbench("release-loss-cut",&filter,1,1).await,Err(CanonicalError::CaptureDraining{..})));
    store.recover_protection_releases().await.unwrap();
    store.result_read_drain.drain().await.unwrap();
    let mut recovered=bounded_query(store.db.query("SELECT state FROM ONLY type::record('canonical_study_captures','release-loss-cut'); SELECT key FROM canonical_study_capture_members WHERE capture='release-loss-cut';")).await.unwrap();
    assert_eq!(recovered.take::<Option<Object>>(0).unwrap().unwrap().get("state"),Some(&Value::String("released".into())));
    assert!(recovered.take::<Vec<Object>>(1).unwrap().is_empty());
    let after=store.canonical_study("release-loss-study").await.unwrap().unwrap();
    assert!(pse_model::SemanticEq::semantic_eq(&before,&after));
}
