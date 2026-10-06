// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit scientific history withdrawal and restartable bounded result cleanup.

use crate::{canonical::{CanonicalError,CanonicalStore,protected_query},canonical_codec};
use surrealdb::types::Object;

/// One bounded cleanup operation after explicit run retirement.
#[derive(Clone,Debug,Default,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultReclamationPage {
    /// All result payloads have been reclaimed; immutable lifecycle receipts remain.
    pub complete:bool,
    /// Original scientific IPC or seed blocks removed in this operation.
    pub batches:u64,
    /// Derived scalar/output indexes or seed descriptors removed.
    pub indexes:u64,
    /// Empty result-set memberships removed.
    pub sets:u64,
}

fn identity(key:&str)->Result<(),CanonicalError> {
    if key.is_empty() || key.len()>4096 {Err(CanonicalError::PayloadLimit)}else{Ok(())}
}

impl CanonicalStore {
    /// Withdraw a terminal study's retention obligation without deleting its
    /// scientific occurrence and outcome receipts. Individual runs are retired
    /// explicitly, and protected readers and retained analyses still prevent it.
    pub async fn forget_study_results(&self,study:&str)->Result<(),CanonicalError> {
        identity(study)?; self.ensure_writes()?;
        protected_query(||Ok(self.db.query("RETURN fn::pse_retention_v1::forget_study($study);").bind(("study",study.to_owned())))).await?;
        Ok(())
    }
    /// Withdraw a derived analysis's result retention. Source roots may then be
    /// released explicitly; the original method and input lineage remain receipts.
    pub async fn forget_analysis_results(&self,analysis:&str)->Result<(),CanonicalError> {
        identity(analysis)?; self.ensure_writes()?;
        protected_query(||Ok(self.db.query("RETURN fn::pse_retention_v1::forget_analysis($analysis);").bind(("analysis",analysis.to_owned())))).await?;
        Ok(())
    }
    /// Irreversibly withdraw scientific payloads only after recovery is complete
    /// and no live reader, retained study or analysis needs them. This atomically
    /// fences future claims/reads and releases every selected run source root.
    /// Repeating it resumes the same retirement after an uncertain acknowledgment.
    pub async fn forget_run_results(&self,run:&str)->Result<(),CanonicalError> {
        identity(run)?;self.ensure_writes()?;
        protected_query(||Ok(self.db.query("RETURN fn::pse_retention_v1::forget_run($run);").bind(("run",run.to_owned())))).await?;
        Ok(())
    }
    /// Reclaim at most one payload and 64 indexes of each supported class. A
    /// persistent generation cursor preserves progress across interruption.
    /// No elapsed-age policy deletes deliberately retained scientific history.
    pub async fn reclaim_result_page(&self,run:&str)->Result<ResultReclamationPage,CanonicalError> {
        identity(run)?;self.ensure_writes()?;
        let mut response=protected_query(||Ok(self.db.query("RETURN fn::pse_retention_v1::collect_run($run);").bind(("run",run.to_owned())))).await?;
        let mut row=response.take::<Option<Object>>(0)?.ok_or(CanonicalError::IncompleteResponse)?;
        Ok(ResultReclamationPage {
            complete:canonical_codec::decode_boolean(canonical_codec::required(&mut row,"complete")?)?,
            batches:canonical_codec::decode_uint(canonical_codec::required(&mut row,"batches")?)?,
            indexes:canonical_codec::decode_uint(canonical_codec::required(&mut row,"indexes")?)?,
            sets:canonical_codec::decode_uint(canonical_codec::required(&mut row,"sets")?)?,
        })
    }
}

#[cfg(all(test,feature="canonical-tests"))]
#[allow(unsafe_code,reason="controlled storage fixture supplies explicit scientific terminal classification")]
mod canonical_result_retention_server_unit {
    use super::*;
    use crate::{canonical::{CanonicalOptions,checked},canonical_execution::{RunRequest,TerminalClass},generated::surreal as wire};
    use std::{path::Path,time::Duration};

    async fn fixture()->(CanonicalStore,String) {
        let state=std::env::var("PSE_SURREAL_STATE").expect("supervised canonical fixture required");
        let mut options=CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database=format!("result_retention_{}",uuid::Uuid::new_v4().simple());
        let store=CanonicalStore::connect(&options).await.unwrap();store.create().await.unwrap();
        (store,options.database)
    }
    async fn request(store:&CanonicalStore,key:&str)->RunRequest {
        RunRequest{key:key.into(),revision:store.edit(key,None,&format!("source-{key}"),&[]).await.unwrap(),sources:vec![],request:vec![1],source_selection:vec![2],attestation:vec![3]}
    }
    async fn finish(store:&CanonicalStore,request:&RunRequest)->String {
        store.begin_run(request).await.unwrap();
        let fence=store.claim_run(&request.key,&format!("claim-{}",request.key),"worker",Duration::from_secs(60)).await.unwrap();
        for ordinal in 0..3 {
            store.append_result_batch(&fence,&format!("append-{}-{ordinal}",request.key),"values",ordinal,&[ordinal as u8,17],1).await.unwrap();
        }
        let closed=store.close_result_ingestion(&fence,&format!("close-{}",request.key)).await.unwrap();
        let manifest=store.reconcile_closed_attempt(&closed).await.unwrap();
        unsafe{store.seal_attempt(&manifest,&format!("seal-{}",request.key),TerminalClass::Partial,&[42]).await}.unwrap();
        fence.attempt().into()
    }
    async fn cleanup(store:&CanonicalStore,run:&str)->(u64,u64) {
        let mut total=(0,0);
        for _ in 0..32 {
            let page=store.reclaim_result_page(run).await.unwrap();total.0+=page.batches;total.1+=page.sets;
            if page.complete {return total;}
        }
        panic!("bounded fixture cleanup did not finish")
    }
    #[tokio::test]
    async fn retired_results_respect_readers_resume_cleanup_and_preserve_receipts() {
        let (store,database)=fixture().await;
        let mut request=request(&store,"retained").await;
        request.sources.push(store.edit("physical",None,"physical-1",&[]).await.unwrap());
        let attempt=finish(&store,&request).await;
        let read=store.read_results(&request.key,&attempt,Duration::from_secs(60)).await.unwrap();
        let payload=store.result_payload(&read,&read.sets()[0].key,0).await.unwrap();
        assert!(store.forget_run_results(&request.key).await.is_err());
        drop(read);
        assert!(store.forget_run_results(&request.key).await.is_err());
        assert_eq!(payload.batch.payload.as_slice(),[0,17]);
        drop(payload);
        // The last returned buffer releases its admitted protection asynchronously.
        let mut retired=false;
        for _ in 0..32 {
            if store.forget_run_results(&request.key).await.is_ok() {retired=true;break;}
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(retired);
        assert!(store.read_results(&request.key,&attempt,Duration::from_secs(60)).await.is_err());
        assert!(store.begin_run(&request).await.is_err());
        assert!(store.claim_run(&request.key,"claim-retained","worker",Duration::from_secs(60)).await.is_err());
        assert_eq!(store.reclaim_result_page(&request.key).await.unwrap().batches,1);
        // A reopened handle continues the recorded cleanup cursor without re-solving.
        assert_eq!(cleanup(&store.clone(),&request.key).await,(2,1));
        assert!(store.reclaim_result_page(&request.key).await.unwrap().complete);
        assert_eq!(store.canonical_attempt(&attempt).await.unwrap().unwrap().outcome.as_deref(),Some("partial"));
        assert!(store.canonical_attempt(&attempt).await.unwrap().unwrap().completion.is_none());
        let mut roots=store.db.query("SELECT key FROM canonical_roots WHERE owner_kind='run' AND owner=$run;").bind(("run",request.key.clone())).await.and_then(checked).unwrap();
        assert!(roots.take::<Vec<Object>>(0).unwrap().is_empty());
        assert_eq!(store.canonical_run(&request.key).await.unwrap().unwrap().interpretation,wire::INTERPRETATION);
        store.db.query(format!("REMOVE DATABASE {database};")).await.and_then(checked).unwrap();
    }
    #[tokio::test]
    async fn result_read_and_retirement_conflict_on_exact_selection() {
        let (store,database)=fixture().await;
        for index in 0..8 {
            let request=request(&store,&format!("race-{index}")).await;
            let attempt=finish(&store,&request).await;
            let (read,retired)=tokio::join!(store.read_results(&request.key,&attempt,Duration::from_secs(60)),store.forget_run_results(&request.key));
            assert!(!(read.is_ok()&&retired.is_ok()));
            if let Ok(read)=read {
                assert_eq!(store.result_payload(&read,&read.sets()[0].key,0).await.unwrap().batch.payload.as_slice(),[0,17]);
                drop(read);
                for _ in 0..32 {
                    if store.forget_run_results(&request.key).await.is_ok(){break;}
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
            assert_eq!(cleanup(&store,&request.key).await,(3,1));
        }
        store.db.query(format!("REMOVE DATABASE {database};")).await.and_then(checked).unwrap();
    }
}
