// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual connection-only policy controls in a private owned native deployment.
use crate::{
    canonical_control::{
        ControlCancelRequest, ControlCommandStatus, ControlReconcileRequest, StudyWorkbenchCursor,
        StudyWorkbenchRequest,
    },
    canonical_study_workbench::StudyWorkbenchFilter,
};

/// A handler error converted to an API response must still poison the complete
/// caller transaction, including a write made before entering the handler.
#[tokio::test]
async fn native_api_failure_rolls_back_earlier_enclosing_write_but_409_commits()
-> Result<(), crate::canonical::CanonicalError> {
    use crate::canonical::bounded_query;
    use surrealdb_types::Object;
    let store = crate::native_fixture::canonical_fixture_store()?;
    bounded_query(store.db.query(r#"
        DEFINE API '/pse-test-failure' FOR get PERMISSIONS FULL THEN {
            CREATE canonical_guards:api_handler_failure SET key='api_handler_failure',generation=0u64;
            THROW 'controlled handler failure';
        };
        DEFINE API '/pse-test-refusal' FOR get PERMISSIONS FULL THEN {
            CREATE canonical_guards:api_handler_refusal SET key='api_handler_refusal',generation=0u64;
            RETURN {status:409,body:{refused:true}};
        };
    "#)).await?;
    let failed = bounded_query(store.db.query(
        r#"
        CREATE canonical_guards:api_earlier_failure SET key='api_earlier_failure',generation=0u64;
        RETURN api::invoke('/pse-test-failure',{method:'get'});
    "#,
    ))
    .await;
    assert!(
        failed.is_err(),
        "API error cannot authorize the enclosing commit"
    );
    for key in ["api_earlier_failure", "api_handler_failure"] {
        let mut reply = bounded_query(
            store
                .db
                .metadata_read_query(
                    crate::canonical::MetadataResponse::Header,
                    "SELECT * FROM ONLY type::record('canonical_guards',$key);",
                )
                .bind(("key", key)),
        )
        .await?;
        assert!(reply.take::<Option<Object>>(0)?.is_none());
    }
    bounded_query(store.db.query(
        r#"
        CREATE canonical_guards:api_earlier_refusal SET key='api_earlier_refusal',generation=0u64;
        RETURN api::invoke('/pse-test-refusal',{method:'get'});
    "#,
    ))
    .await?;
    for key in ["api_earlier_refusal", "api_handler_refusal"] {
        let mut reply = bounded_query(
            store
                .db
                .metadata_read_query(
                    crate::canonical::MetadataResponse::Header,
                    "SELECT * FROM ONLY type::record('canonical_guards',$key);",
                )
                .bind(("key", key)),
        )
        .await?;
        assert!(reply.take::<Option<Object>>(0)?.is_some());
    }
    store.remove_isolated_fixture().await?;
    Ok(())
}

#[tokio::test]
async fn native_control_browse_and_original_refused_command_require_no_math()
-> Result<(), crate::canonical::CanonicalError> {
    let store = crate::native_fixture::canonical_fixture_store()?;
    let filter = StudyWorkbenchFilter::default();
    let page = store
        .control_study_workbench_page(StudyWorkbenchRequest {
            filter: filter.clone(),
            maximum: 1,
            cursor: None,
        })
        .await?;
    assert!(page.headers.is_empty());
    assert!(page.next.is_none());
    assert!(
        store
            .control_study_workbench_page(StudyWorkbenchRequest {
                filter: filter.clone(),
                maximum: 0,
                cursor: None,
            })
            .await
            .is_err()
    );
    assert!(
        store
            .control_study_workbench_page(StudyWorkbenchRequest {
                filter: filter.clone(),
                maximum: 1,
                cursor: Some(StudyWorkbenchCursor {
                    issuing_store: uuid::Uuid::new_v4().to_string(),
                    filter,
                    after: "opaque-after".into(),
                }),
            })
            .await
            .is_err()
    );
    let request = ControlCancelRequest::issue(
        store.issuing_store()?.to_owned(),
        "opaque:absent-0001".into(),
    );
    let original =
        crate::testing::lose_control_cancel_ack(false, store.cancel_study_command(&request))
            .await?;
    assert_eq!(original.status, ControlCommandStatus::Refused);
    let repeated = store.cancel_study_command(&request).await?;
    assert_eq!(
        serde_json::to_value(&original).unwrap(),
        serde_json::to_value(&repeated).unwrap()
    );
    let readback = store
        .reconcile_study_cancel(&ControlReconcileRequest {
            document_version: Default::default(),
            issuing_store: request.issuing_store.clone(),
            operation_id: request.operation_id.clone(),
        })
        .await?;
    assert_eq!(
        serde_json::to_value(readback.receipt.unwrap()).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    let mut changed = request.clone();
    changed.study = "different:absent-0001".into();
    assert!(matches!(
        store.cancel_study_command(&changed).await,
        Err(crate::canonical::CanonicalError::OperationReused)
    ));
    let absent = store
        .reconcile_study_cancel(&ControlReconcileRequest {
            document_version: Default::default(),
            issuing_store: request.issuing_store.clone(),
            operation_id: uuid::Uuid::new_v4().to_string(),
        })
        .await?;
    assert!(absent.receipt.is_none());
    let uncertain = ControlCancelRequest::issue(
        request.issuing_store.clone(),
        "unobserved-original-target".into(),
    );
    let error =
        crate::testing::lose_control_cancel_ack(true, store.cancel_study_command(&uncertain))
            .await
            .unwrap_err();
    assert!(error.effect_is_unknown());
    match error {
        crate::canonical::CanonicalError::SettlementUnresolved {
            owner,
            operation,
            request: intent,
            readback: Some(_),
            ..
        } => {
            assert_eq!(owner, "control-study-cancel");
            assert_eq!(operation, uncertain.operation_id);
            assert_eq!(intent, serde_json::to_vec(&uncertain).unwrap());
        }
        error => panic!("lost ACK/readback lost original uncertainty: {error:?}"),
    }
    let committed = store
        .reconcile_study_cancel(&ControlReconcileRequest {
            document_version: Default::default(),
            issuing_store: uncertain.issuing_store.clone(),
            operation_id: uncertain.operation_id.clone(),
        })
        .await?
        .receipt
        .expect("actual original cancellation receipt committed");
    assert_eq!(committed.operation_id, uncertain.operation_id);
    assert_eq!(committed.study, uncertain.study);
    let repeated = store.cancel_study_command(&uncertain).await?;
    assert_eq!(
        serde_json::to_value(committed).unwrap(),
        serde_json::to_value(repeated).unwrap()
    );
    let mut foreign = request;
    foreign.issuing_store = uuid::Uuid::new_v4().to_string();
    assert!(store.cancel_study_command(&foreign).await.is_err());
    store.remove_isolated_fixture().await
}
