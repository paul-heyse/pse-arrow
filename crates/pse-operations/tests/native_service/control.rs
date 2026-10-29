// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual connection-only policy controls in a private owned native deployment.
use super::*;
use pse_operations::{
    canonical_control::{
        ControlCancelRequest, ControlCommandStatus, ControlReconcileRequest, StudyWorkbenchCursor,
        StudyWorkbenchRequest,
    },
    canonical_study_workbench::StudyWorkbenchFilter,
};

#[tokio::test]
async fn native_control_browse_and_original_refused_command_require_no_math() {
    let mut service = fixture_with_schema(false).unwrap();
    service
        .run(Instant::now() + Duration::from_secs(180), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let store = service.ready()?.store().clone();
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
                let original = pse_operations::testing::lose_control_cancel_ack(
                    false,
                    store.cancel_study_command(&request),
                )
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
                    Err(pse_operations::canonical::CanonicalError::OperationReused)
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
                let error = pse_operations::testing::lose_control_cancel_ack(
                    true,
                    store.cancel_study_command(&uncertain),
                )
                .await
                .unwrap_err();
                assert!(error.effect_is_unknown());
                match error {
                    pse_operations::canonical::CanonicalError::SettlementUnresolved {
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
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
    assert!(!service.state().exists());
}
