// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original queued-study custody across a lost acknowledgement and frontend retirement.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "native original-identity controls require valid fixture setup"
)]

use super::*;
use crate::host_operations::engineering_contract::{
    EngineeringPrepareStudy, EngineeringRestoreStudy, EngineeringStudyKey, EngineeringSubmitStudy,
};

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn engineering_queued_study_lost_ack_survives_frontend_retirement_and_refuses_rebinding() {
    use crate::workflow::{
        CaseOperation, StudyInputs, StudyOutputDemand, StudyPoint, StudyPointPolicy, StudyRequest,
    };
    use pse_model::{
        document::Version,
        generated::enums::ModelingAnalysisRoute,
        study::{OccurrenceKey, StartPolicy},
    };

    let store = pse_operations::native_fixture::canonical_fixture_store().unwrap();
    let (service, _spill) = tests::service_on(Some(store.clone()));
    let source = tests::admitted(&service).await;
    let page = service
        .inspect_source(
            EngineeringInspectSource {
                operation: source.operation.clone(),
                offset: 0,
                limit: 128,
            },
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let case = page
        .declarations
        .iter()
        .find(|declaration| declaration.name == "square")
        .unwrap()
        .declaration;
    let preparation = service
        .prepare_study(EngineeringPrepareStudy {
            operation: "prepared-original-study".into(),
            source_operation: source.operation,
            expected_source: source.local_revision,
            request: StudyRequest {
                version: Version,
                output: StudyOutputDemand::Full {},
                inputs: StudyInputs::Explicit(vec![StudyPoint {
                    operation: crate::workflow::OperationRequest::DeclaredCase(CaseOperation {
                        case: case.into(),
                        route: ModelingAnalysisRoute::Steady,
                        settings: crate::math::settings::SolveSettings {
                            intent: pse_backend_native::solve::SolveIntent::Root,
                            backend: Some(pse_backend_native::solve::Backend::Kinsol),
                            ..Default::default()
                        },
                    }),
                    preparation: PreparationSettings {
                        compiler: crate::workflow::tests::compiler_profile(),
                        ..Default::default()
                    },
                    overlay: crate::workflow::PointOverlay::default(),
                    policy: StudyPointPolicy {
                        key: OccurrenceKey(7),
                        dependencies: Vec::new(),
                        start: StartPolicy::Fresh,
                        attempt_limit: 1,
                    },
                }]),
            },
        })
        .await
        .unwrap();
    let retained = service
        .prepared_study_definition(&EngineeringOperationKey {
            operation: preparation.operation.clone(),
        })
        .await
        .unwrap();
    let submitted = EngineeringSubmitStudy {
        operation: "original-issued-study".into(),
        preparation_operation: preparation.operation.clone(),
        expected_preparation: preparation.preparation,
        study: pse_operations::mint_id(),
    };
    let key = EngineeringStudyKey {
        operation: submitted.operation.clone(),
        study: submitted.study,
    };
    let waiter = {
        let service = service.clone();
        let submitted = submitted.clone();
        tokio::spawn(async move { service.submit_study(submitted).await })
    };
    let original = tokio::time::timeout(std::time::Duration::from_secs(90), async {
        loop {
            if let Ok(original) = service.operation(&submitted.operation, Kind::Study) {
                break original;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    waiter.abort();
    let _lost_acknowledgement = waiter.await;
    assert!(matches!(
        original.completed().await.unwrap(),
        Outcome::Study(_)
    ));
    let before = service
        .study_status(&key)
        .await
        .unwrap()
        .value
        .canonical
        .unwrap();
    assert!(before.active);
    assert!(!before.cancelled);
    service.shutdown().await.unwrap();
    assert!(
        !store
            .canonical_study_summary(&submitted.study.to_string())
            .await
            .unwrap()
            .unwrap()
            .cancelled
    );

    let (reader, _reader_spill) = tests::service_on(Some(store.clone()));
    let recovered = reader.submit_study(submitted.clone()).await.unwrap();
    assert_eq!(recovered.value.canonical.as_ref(), Some(&before));
    // Local retention names can be reused in a new frontend. Their current data must
    // never change the meaning of the original persisted submit envelope.
    let mut changed_definition = retained.value.clone();
    changed_definition.points[0].policy.attempt_limit += 1;
    let changed_identity =
        pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &changed_definition).unwrap();
    assert_ne!(changed_identity, submitted.expected_preparation);
    reader
        .restore_study(EngineeringRestoreStudy {
            operation: preparation.operation,
            expected_preparation: changed_identity,
            definition: changed_definition,
        })
        .await
        .unwrap();
    assert!(
        reader
            .submit_study(EngineeringSubmitStudy {
                expected_preparation: changed_identity,
                ..submitted.clone()
            })
            .await
            .is_err()
    );
    assert_eq!(
        reader
            .submit_study(submitted)
            .await
            .unwrap()
            .value
            .canonical
            .as_ref(),
        Some(&before)
    );
    reader.cancel_study(&key).await.unwrap();
    assert!(
        reader
            .study_status(&key)
            .await
            .unwrap()
            .value
            .canonical
            .unwrap()
            .cancelled
    );
    reader.shutdown().await.unwrap();
}
