// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
fn snapshot() -> EvidenceSnapshot {
    EvidenceSnapshot {
        version: 2,
        authority: "original-host-owner".into(),
        codec: "exact-resource-files-invocation-v2".into(),
        source_id: "invocation-1".into(),
        snapshot_digest: "a".repeat(64),
        coverage: "partial".into(),
        expected_members: vec!["invocation".into(), "binary::unreported".into()],
        facts: vec![
            EvidenceFact {
                member: "invocation".into(),
                kind: "invocation".into(),
                status: "command_failed".into(),
                crate_name: None,
                test_id: None,
                detail_ref: Some("owner:invocation-1".into()),
                ..EvidenceFact::default()
            },
            EvidenceFact {
                member: "binary::unreported".into(),
                kind: "rust_test".into(),
                status: "unreported".into(),
                crate_name: Some("pse-example".into()),
                test_id: Some("binary::unreported".into()),
                detail_ref: None,
                ..EvidenceFact::default()
            },
        ],
    }
}
#[test]
fn evidence_exact_membership_and_canonical_import_identity() {
    let source = snapshot();
    source.validate().unwrap();
    let id = source.identity().unwrap();
    let mut reordered = source.clone();
    reordered.facts.reverse();
    reordered.expected_members.reverse();
    assert_eq!(id, reordered.identity().unwrap());
    let mut missing = source.clone();
    missing.facts.pop();
    assert!(missing.validate().is_err());
    let mut duplicate = source.clone();
    duplicate.facts.push(duplicate.facts[0].clone());
    assert!(duplicate.validate().is_err());
    let mut oversized = source.clone();
    oversized.authority = "a".repeat(4097);
    assert!(oversized.validate().is_err());
    let mut changed = source;
    changed.facts[0].status = "failed".into();
    let changed = changed.identity().unwrap();
    assert_eq!(id.0, changed.0);
    assert_ne!(id.1, changed.1);
}
#[test]
fn evidence_generated_codec_retains_exact_uint64_and_rejects_foreign_record() {
    let row = pse_model::generated::runtime::evidence_generations::Row {
        key: "g".into(),
        authority: "a".into(),
        codec: "c".into(),
        source_id: "s".into(),
        snapshot_digest: "a".repeat(64),
        content_digest: "b".repeat(64),
        coverage: pse_model::generated::enums::EvidenceCoverage::Partial,
        expected_members: u64::MAX,
        state: pse_model::generated::enums::EvidenceGenerationState::Staged,
    };
    let mut object = wire::encode_evidence_generations(&row).unwrap();
    assert_eq!(
        wire::decode_evidence_generations(object.clone())
            .unwrap()
            .expected_members,
        u64::MAX
    );
    let mut bad_coverage = object.clone();
    bad_coverage.insert("coverage", Value::String("contradictory".into()));
    assert!(wire::decode_evidence_generations(bad_coverage).is_err());
    let mut bad_state = object.clone();
    bad_state.insert("state", Value::String("unfinished_but_complete".into()));
    assert!(wire::decode_evidence_generations(bad_state).is_err());
    object.insert(
        "id",
        crate::canonical_codec::record_link("canonical_runs", "g"),
    );
    assert!(wire::decode_evidence_generations(object).is_err());
}
#[test]
fn evidence_config_debug_does_not_disclose_secrets() {
    let options = ToolingOptions {
        endpoint: "grpc://127.0.0.1:8000".into(),
        namespace: "pse_tooling".into(),
        database: "evidence".into(),
        service_instance_id: "service-1".into(),
        credentials: Credentials {
            reader_username: "reader".into(),
            reader_password: "secret-reader".into(),
            importer_username: "importer".into(),
            importer_password: "secret-importer".into(),
        },
    };
    assert!(!format!("{options:?}").contains("secret"));
    assert!(options.address().is_ok());
    let mut foreign = options;
    foreign.endpoint = "grpc://192.0.2.1:8000".into();
    assert!(foreign.address().is_err());
}
#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn evidence_native_scoped_roles_complete_generation_and_cursor_fences() {
    let state = std::path::PathBuf::from(
        std::env::var("PSE_SURREAL_STATE").expect("explicit selected service required"),
    );
    let deployment: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("config.json")).unwrap()).unwrap();
    let credentials: serde_json::Value = serde_json::from_slice(
        &std::fs::read(deployment["credentials_file"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    let username = credentials["username"].as_str().unwrap();
    let password = credentials["password"].as_str().unwrap();
    let options = ToolingOptions {
        endpoint: deployment["endpoint"].as_str().unwrap().into(),
        namespace: format!("pse_tooling_test_{}", uuid::Uuid::new_v4().simple()),
        database: "evidence".into(),
        service_instance_id: deployment["instance_id"].as_str().unwrap().into(),
        credentials: Credentials {
            reader_username: "reader".into(),
            reader_password: uuid::Uuid::new_v4().to_string(),
            importer_username: "importer".into(),
            importer_password: uuid::Uuid::new_v4().to_string(),
        },
    };
    eprintln!("owned evidence fixture namespace {}", options.namespace);
    assert!(
        EvidenceStore::open(&options, EvidenceAccess::Reader)
            .await
            .is_err()
    );
    EvidenceStore::initialize(&options, username, password)
        .await
        .unwrap();
    use futures_util::FutureExt as _;
    let result=std::panic::AssertUnwindSafe(async{
        let importer=EvidenceStore::open(&options,EvidenceAccess::Importer).await?;
        let reader=EvidenceStore::open(&options,EvidenceAccess::Reader).await?;
        assert!(reader.import(&snapshot()).await.is_err());
        let _refused=canonical::bounded_query(reader.query("CREATE evidence_facts:forbidden SET key='forbidden';")?).await;
        let mut absence=canonical::bounded_query(importer.query("SELECT * FROM evidence_facts:forbidden;")?).await?;
        assert!(absence.take::<Vec<Object>>(0)?.is_empty(),"read-only principal cannot create facts even when native permissions return an empty selection");
        assert!(canonical::bounded_query(importer.query("DEFINE USER forbidden ON DATABASE PASSWORD 'not-a-secret' ROLES OWNER;")?).await.is_err());
        let service_ns=deployment["namespace"].as_str().unwrap();let service_db=deployment["database"].as_str().unwrap();
        for (user,pass) in [(&options.credentials.reader_username,&options.credentials.reader_password),(&options.credentials.importer_username,&options.credentials.importer_password)] {
            let probe=EvidenceStore::client(&options).await?;
            probe.signin(surrealdb::opt::auth::Database{namespace:options.namespace.clone(),database:options.database.clone(),username:user.clone(),password:pass.clone()}).request_context(RequestContext::control(original_deadline(canonical::REQUEST_TIMEOUT).into_std())).await?;
            let selected=probe.use_ns(service_ns).use_db(service_db).request_context(RequestContext::control(original_deadline(canonical::REQUEST_TIMEOUT).into_std())).await;
            if selected.is_ok() && let Ok(mut response)=canonical::bounded_query(probe.query("SELECT * FROM canonical_interpretations:current;").request_context(RequestContext::new(original_deadline(canonical::REQUEST_TIMEOUT).into_std()))).await{assert!(response.take::<Vec<Object>>(0)?.is_empty(),"database scoped account must not read canonical authority");}
            probe.disconnect().await?;
        }
        let source=snapshot();importer.ack_fault.store(1,std::sync::atomic::Ordering::SeqCst);let first=importer.import(&source).await?;let repeated=importer.import(&source).await?;assert_eq!(first.generation,repeated.generation);
        crate::knowledge::tests::exercise_native(&importer,&reader).await?;
        let query=EvidenceQuery{limit:Some(1),..Default::default()};let page=reader.select(&query).await?;assert_eq!(page.facts.len(),1);assert_eq!(page.coverage,"partial");let cursor=page.next.unwrap();
        let complete=reader.select(&EvidenceQuery{cursor:Some(cursor.clone()),..query.clone()}).await?;assert_eq!(complete.facts.len(),1);assert!(complete.next.is_none());
        let mut wrong=cursor.clone();wrong.issuing_store="foreign".into();assert!(matches!(reader.select(&EvidenceQuery{cursor:Some(wrong),..query.clone()}).await,Err(EvidenceError::Cursor)));
        let mut changed=source.clone();changed.facts[0].status="changed".into();assert!(matches!(importer.import(&changed).await,Err(EvidenceError::Reused)));
        let mut later=source.clone();later.source_id="invocation-2".into();importer.ack_fault.store(2,std::sync::atomic::Ordering::SeqCst);let second=importer.import(&later).await?;assert_ne!(first.generation,second.generation);
        assert!(matches!(reader.select(&EvidenceQuery{cursor:Some(cursor),..query.clone()}).await,Err(EvidenceError::Cursor)));
        let b_page=reader.select(&query).await?;let b_cursor=b_page.next.unwrap();
        importer.ack_fault.store(2,std::sync::atomic::Ordering::SeqCst);
        let settled_a=importer.import(&source).await?;assert_eq!(settled_a.generation,first.generation);
        let continued_b=reader.select(&EvidenceQuery{cursor:Some(b_cursor),..query}).await?;
        assert_eq!(continued_b.generation,second.generation,"replaying activated A must preserve B and its cursor");
        assert_eq!(reader.status().await?["active_generation"],second.generation);
        let old=reader.select(&EvidenceQuery{generation:Some(first.generation.clone()),..Default::default()}).await?;assert_eq!(old.facts.len(),2);
        let cut=reader.source_cut(&[first.generation.clone(),second.generation.clone(),first.generation.clone()],false).await?;
        assert_eq!(cut.len(),2,"explicit multi-source membership deduplicates repeated generation IDs");
        #[cfg(feature="evidence-query")]
        {
            let mut offline=crate::evidence_query::EvidenceAnalysis::new(vec![source.clone(),later.clone()],false)?;
            let mut indexed=crate::evidence_query::EvidenceAnalysis::new(cut,false)?;
            let question=crate::evidence_query::EvidenceQuestion::default();
            assert_eq!(offline.ask(&question,&[]).await?,indexed.ask(&question,&[]).await?,"same selected facts give offline/indexed parity");
        }
        let mut version=source.clone();version.snapshot_digest="d".repeat(64);version.facts[1].status="pass".into();
        let changed_receipt=importer.import(&version).await?;
        assert!(reader.source_cut(&[first.generation.clone(),changed_receipt.generation.clone()],false).await.is_err());
        assert_eq!(reader.source_cut(&[first.generation.clone(),changed_receipt.generation],true).await?.len(),2);
        let history=reader.generations(None,64).await?;assert_eq!(history["generations"].as_array().unwrap().len(),3);
        #[cfg(feature="evidence-query")]
        crate::evidence_query::exercise_indexed_comparison(&importer,&reader).await?;
        // A staged generation with no complete membership never becomes selected.
        let (key,digest)= {let mut interrupted=source.clone();interrupted.source_id="interrupted".into();interrupted.identity()?};
        let row=pse_model::generated::runtime::evidence_generations::Row{key:key.clone(),authority:"owner".into(),codec:"owner-v1".into(),source_id:"interrupted".into(),snapshot_digest:"c".repeat(64),content_digest:digest,coverage:pse_model::generated::enums::EvidenceCoverage::Partial,expected_members:2,state:pse_model::generated::enums::EvidenceGenerationState::Staged};
        canonical::bounded_query(importer.query("CREATE type::record('evidence_generations',$key) CONTENT $row;")?.bind(("key",key.clone())).bind(("row",wire::encode_evidence_generations(&row)?))).await?;
        assert!(matches!(reader.select(&EvidenceQuery{generation:Some(key.clone()),..Default::default()}).await,Err(EvidenceError::Cursor)));
        assert!(matches!(reader.source_cut(&[key],false).await,Err(EvidenceError::Unavailable(_))));
        let mut invalid=options.clone();invalid.service_instance_id="foreign-service".into();assert!(matches!(EvidenceStore::open(&invalid,EvidenceAccess::Reader).await,Err(EvidenceError::Unavailable(_))));
        // Complete-response checking rejects preceding rows when a later statement fails.
        assert!(canonical::bounded_query(importer.query("SELECT * FROM evidence_facts LIMIT 1; THROW 'later statement failure';")?).await.is_err());
        // Pinned native REMOVE DATABASE remains inside its enclosing transaction.
        let admin=EvidenceStore::client(&options).await?;let context=||RequestContext::control(original_deadline(canonical::REQUEST_TIMEOUT).into_std());
        admin.signin(surrealdb::opt::auth::Root{username:username.into(),password:password.into()}).request_context(context()).await?;
        admin.use_ns(&options.namespace).use_db(&options.database).request_context(context()).await?;
        let cancelled=canonical::bounded_query(admin.query(format!("BEGIN; SELECT * FROM evidence_interpretations:current; REMOVE DATABASE `{}`; CANCEL;",options.database)).request_context(context())).await;
        assert!(matches!(&cancelled,Err(CanonicalError::Driver(error)) if matches!(error.query_details(),Some(surrealdb::types::QueryError::Cancelled))),"CANCEL must establish rollback, not an earlier failure: {cancelled:?}");admin.disconnect().await?;
        let retained=EvidenceStore::open(&options,EvidenceAccess::Reader).await?;assert_eq!(retained.issuing_store,reader.issuing_store);retained.disconnect().await?;
        let entered=Arc::new(tokio::sync::Notify::new());let resume=Arc::new(tokio::sync::Notify::new());
        DROP_PAUSE.with(|pause|*pause.borrow_mut()=Some((entered.clone(),resume.clone())));
        let pending_options=options.clone();let pending_username=username.to_owned();let pending_password=password.to_owned();
        let stale_delete=tokio::spawn(async move{EvidenceStore::drop_database(&pending_options,&pending_username,&pending_password).await});
        entered.notified().await;
        EvidenceStore::drop_database(&options,username,password).await?;
        EvidenceStore::initialize(&options,username,password).await?;
        resume.notify_one();match stale_delete.await.unwrap(){Err(EvidenceError::LifecycleFailure{operation,issuing_store,..})=>{assert_eq!(operation,"drop");assert_eq!(issuing_store,reader.issuing_store);},other=>panic!("expected an issuer-bound stale deletion refusal, observed {other:?}")};
        let replacement=EvidenceStore::open(&options,EvidenceAccess::Reader).await?;
        assert_ne!(replacement.issuing_store,reader.issuing_store);replacement.status().await?;replacement.disconnect().await?;
        assert!(matches!(reader.status().await,Err(EvidenceError::Unavailable(_))),"old reader must not adopt a replacement issuing identity");
        assert!(matches!(importer.generations(None,64).await,Err(EvidenceError::Unavailable(_))),"history selection must retain the original issuing authority");
        reader.disconnect().await?;importer.disconnect().await?;Ok::<_,EvidenceError>(())
    }).catch_unwind().await;
    EvidenceStore::drop_database(&options, username, password)
        .await
        .unwrap();
    let admin = EvidenceStore::client(&options).await.unwrap();
    admin
        .signin(surrealdb::opt::auth::Root {
            username: username.into(),
            password: password.into(),
        })
        .request_context(RequestContext::control(
            original_deadline(canonical::REQUEST_TIMEOUT).into_std(),
        ))
        .await
        .unwrap();
    admin
        .query(format!("REMOVE NAMESPACE `{}`;", options.namespace))
        .request_context(RequestContext::control(
            original_deadline(canonical::REQUEST_TIMEOUT).into_std(),
        ))
        .await
        .unwrap()
        .check()
        .unwrap();
    admin.disconnect().await.unwrap();
    match result {
        Ok(result) => result.unwrap(),
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

#[test]
fn evidence_foreign_codec_context_and_sample_membership_refuse() {
    let mut source = snapshot();
    source.version = 1;
    assert!(source.validate().is_err());
    source.version = 2;
    source.codec = "foreign-codec".into();
    assert!(source.validate().is_err());
    let mut source = snapshot();
    source.facts[1].context_id = Some("undeclared".into());
    assert!(source.validate().is_err());
    let mut source = snapshot();
    source.facts[1].value_ns = Some(f64::NAN);
    assert!(source.validate().is_err());
}
