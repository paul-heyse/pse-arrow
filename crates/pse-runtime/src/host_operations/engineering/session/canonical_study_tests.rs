// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original queued Study custody across acknowledgement loss, stream reset and frontend retirement.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "native original-identity controls require valid fixture setup"
)]

use super::*;
use crate::host_operations::engineering_contract::{
    EngineeringPrepareExplicitStudy, EngineeringPreparedStudy, EngineeringRestoreExplicitStudy,
    EngineeringStudyKey, EngineeringSubmitStudy,
};

#[cfg(feature = "solver-kinsol")]
async fn prepare_queued_study(service: &Arc<EngineeringService>) -> EngineeringPreparedStudy {
    use crate::workflow::{CaseOperation, StudyOutputDemand, StudyPoint, StudyPointPolicy};
    use pse_model::{
        generated::enums::ModelingAnalysisRoute,
        study::{OccurrenceKey, StartPolicy},
    };

    let source = tests::admitted(service).await;
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
    service
        .prepare_explicit_study(EngineeringPrepareExplicitStudy {
            operation: "prepared-original-study".into(),
            source_operation: source.operation,
            expected_source: source.local_revision,
            output: StudyOutputDemand::Full {},
            points: vec![StudyPoint {
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
            }],
        })
        .await
        .unwrap()
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn engineering_queued_study_lost_ack_survives_frontend_retirement_and_refuses_rebinding() {
    use crate::workflow::StudyDefinition;

    let store = pse_operations::native_fixture::canonical_fixture_store().unwrap();
    let (service, _spill) = tests::service_on(Some(store.clone()));
    let preparation = prepare_queued_study(&service).await;
    let retained = service
        .read_prepared_explicit_study(&EngineeringOperationKey {
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
    let changed_identity = pse_ids::document::of(
        pse_ids::Frame::DurableStudyRequestV5,
        &StudyDefinition::from(changed_definition.clone()),
    )
    .unwrap();
    assert_ne!(changed_identity, submitted.expected_preparation);
    reader
        .restore_explicit_study(EngineeringRestoreExplicitStudy {
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

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn engineering_study_h2_reset_joins_admitted_canonical_capture_and_releases_ingress() {
    use crate::{
        engineering_projection::encode_engineering_observe_study,
        host_operations::{
            engineering::{LocalAccessPolicy, generated::pb::pse::engineering::v1 as pb, serve},
            engineering_contract::EngineeringObserveStudy,
        },
        workflow::{StudyObservationRequest, StudyObserver},
    };
    use buffa::Message as _;
    use bytes::Bytes;
    use http_body_util::{BodyExt as _, Full};
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use pse_operations::canonical::CanonicalError;
    use std::{sync::atomic::{AtomicBool, Ordering}, time::Duration};

    let store = pse_operations::native_fixture::canonical_fixture_store().unwrap();
    let (service, _spill) = tests::service_on(Some(store.clone()));
    let preparation = prepare_queued_study(&service).await;
    let submitted = EngineeringSubmitStudy {
        operation: "original-reset-study".into(),
        preparation_operation: preparation.operation,
        expected_preparation: preparation.preparation,
        study: pse_operations::mint_id(),
    };
    let study = submitted.study.to_string();
    let original = EngineeringStudyKey {
        operation: submitted.operation.clone(),
        study: submitted.study,
    };
    let queued = service.submit_study(submitted).await.unwrap().value.canonical.unwrap();
    assert!(queued.active && !queued.cancelled);
    let observer = encode_engineering_observe_study(EngineeringObserveStudy {
        original,
        request: StudyObservationRequest {
            study: study.clone(),
            maximum_members: 1,
            maximum_events: 1,
        },
        cursor: None,
    }).unwrap();
    let probe = StudyObserver::probe_next_capture(study.clone());

    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("engineering.sock");
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    let stop = CancelSource::new();
    let server_finished = Arc::new(AtomicBool::new(false));
    let native_finished = server_finished.clone();
    let engine = service.clone();
    let serving = async {
        let result = serve(
            listener,
            service,
            LocalAccessPolicy {
                uid: rustix::process::geteuid().as_raw(),
                gid: None,
                maximum_connections: 1,
                connection_drain: Duration::from_secs(5),
            },
            stop.clone(),
        ).await;
        server_finished.store(true, Ordering::Release);
        result
    };
    let stop_after_reply = stop.clone();
    let capture_store = store.clone();
    let exercise = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(30), async {
            let peer = tokio::net::UnixStream::connect(socket).await.unwrap().into_std().unwrap();
            let retained_peer = peer.try_clone().unwrap();
            let peer = tokio::net::UnixStream::from_std(peer).unwrap();
            let (mut client, connection) = hyper::client::conn::http2::handshake(
                TokioExecutor::new(), TokioIo::new(peer),
            ).await.unwrap();
            let task_inventory = engine.clone();
            let exchange = async move {
                let request = |path: &str, payload: Vec<u8>| {
                    let mut frame = Vec::with_capacity(5 + payload.len());
                    frame.push(0);
                    frame.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes());
                    frame.extend_from_slice(&payload);
                    http::Request::post(format!("http://localhost{path}"))
                        .version(http::Version::HTTP_2)
                        .header(http::header::HOST, "localhost")
                        .header(http::header::CONTENT_TYPE, "application/grpc+proto")
                        .header("te", "trailers")
                        .body(Full::new(Bytes::from(frame)))
                        .unwrap()
                };
                client.ready().await.unwrap();
                let response = client.send_request(request(
                    "/pse.engineering.v1.StudyControlV2/ObserveStudy",
                    observer.encode_to_vec(),
                )).await.unwrap();
                assert_eq!(response.status(), http::StatusCode::OK);
                let (capture, ingress) = probe.entered.await.unwrap();
                assert!(capture.complete);
                assert_eq!((capture.header_count, capture.member_count), (1, 1));
                assert!(ingress.strong_count() > 1, "the live stream still owns the original grant");
                assert!(matches!(
                    capture_store.recover_study_capture(&capture.capture).await,
                    Err(CanonicalError::CaptureInUse { capture: key }) if key == capture.capture
                ));

                // The complete request sent END_STREAM. Dropping the still-open
                // response's last receive reference makes the pinned H2 client
                // reset this stream while the same connection remains driven.
                drop(response);
                loop {
                    let owners = ingress.strong_count();
                    assert!(owners > 0, "the admitted native task lost its original ingress grant");
                    if owners == 1 {
                        break;
                    }
                    tokio::task::yield_now().await;
                }
                assert!(matches!(
                    capture_store.recover_study_capture(&capture.capture).await,
                    Err(CanonicalError::CaptureInUse { capture: key }) if key == capture.capture
                ));
                client.ready().await.unwrap();
                let next = client.send_request(request(
                    "/pse.engineering.v1.EngineeringService/DescribeContract",
                    Vec::new(),
                )).await.unwrap();
                let reply = next.into_body().collect().await.unwrap();
                assert_eq!(reply.trailers().unwrap()["grpc-status"], "0");
                let frame = reply.to_bytes();
                assert_eq!(frame[0], 0);
                let length = u32::from_be_bytes(frame[1..5].try_into().unwrap()) as usize;
                assert_eq!(length, frame.len() - 5);
                let contract = pb::EngineeringContract::decode_from_slice(&frame[5..]).unwrap();
                assert_eq!(contract.descriptor.as_slice(), super::super::generated::DESCRIPTORS);

                // No stream-side owner remains; the live original grant belongs
                // to the service's admitted task inventory.
                assert!(!task_inventory.tasks.lock().unwrap().is_empty());
                stop_after_reply.cancel();
                (client, capture, ingress, probe.resume, probe.completed)
            };
            let ((client, capture, ingress, resume, completed), driver) =
                tokio::join!(exchange, connection);
            driver.unwrap();
            drop(client);
            // The transport drains independently while the native task remains
            // paused. Hold the original receive half through any final writes.
            retained_peer.shutdown(std::net::Shutdown::Write).unwrap();
            while !engine.closing.load(Ordering::Acquire) || !engine.tasks.lock().unwrap().is_empty() {
                tokio::task::yield_now().await;
            }
            assert!(!native_finished.load(Ordering::Acquire));
            assert_eq!(ingress.strong_count(), 1);
            resume.send(()).unwrap();
            assert_eq!(completed.await.unwrap(), capture.capture);
            (retained_peer, capture, ingress)
        })
        .await
        .unwrap()
    });
    let joined = async {
        let result = exercise.await;
        stop.cancel();
        result
    };
    let (result, server) = tokio::join!(joined, serving);
    server.unwrap();
    let (retained_peer, capture, ingress) = result.unwrap();
    drop(retained_peer);
    assert_eq!(ingress.strong_count(), 0, "actual native/transport join released the original grant");
    store.join_result_releases().await.unwrap();
    assert!(matches!(
        store.capture_study_workbench(
            &capture.capture,
            &capture.request.filter,
            capture.request.maximum_studies,
            capture.request.maximum_members,
        ).await,
        Err(CanonicalError::OperationReused)
    ));
    let after = store.canonical_study_summary(&study).await.unwrap().unwrap();
    assert!(after.active && !after.cancelled, "observation departure cannot stop the original Study");
}
