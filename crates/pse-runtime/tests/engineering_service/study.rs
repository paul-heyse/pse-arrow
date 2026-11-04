// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Three externally orchestrated phases; this client never launches or adopts a worker.
//! Unlinked until the original service/client graph and keeper are ready.

use super::*;
use pse_runtime::{host_operations::engineering_contract::*, preparation_projection as codec,
    workflow::{CaseOperation, OperationRequest, PointOverlay, PreparationSettings, StudyDefinition,
        StudyInputs, StudyOutputDemand, StudyPoint, StudyPointPolicy, StudyRequest}};
use pse_model::{document::Version, generated::{enums::ModelingAnalysisRoute,
    identities::{DeclarationId, RunId}}, study::{OccurrenceKey, StartPolicy}};
use pse_ids::{ContentHash, SemanticId};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Handoff {
    version: Version<1>,
    keeper: ContentHash,
    frontend: pse_host::envelope::CliInvocationId,
    source: pse_model::engineering::EngineeringSourceSelection,
    submitted: EngineeringSubmitStudy,
    cancelled: EngineeringSubmitStudy,
    definition: StudyDefinition,
    queued: pse_operations::canonical_studies::StudySummary,
    cancelled_queued: pse_operations::canonical_studies::StudySummary,
    retired_capture: String,
    retired_receipt: pse_runtime::workflow::StudyCohortReceipt,
    cursor: pse_runtime::workflow::StudyObservationCursor,
}

fn selected_path(name: &str) -> PathBuf {
    std::env::var_os(name).map(PathBuf::from)
        .unwrap_or_else(|| panic!("explicit original fixture must select {name}"))
}
fn keeper_identity() -> ContentHash {
    let bytes = std::fs::read(selected_path("PSE_ENGINEERING_KEEPER_READY")).unwrap();
    let ready: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(ready["database"].as_str().is_some_and(|name| !name.is_empty()));
    assert!(ready["state"].as_str().is_some_and(|name| !name.is_empty()));
    // A routing receipt identifies the unchanged fixture; it grants no performing custody.
    pse_ids::encoding_checksum(&bytes).content_hash()
}
fn gate_is(open: bool) {
    assert_eq!(selected_path("PSE_ENGINEERING_STUDY_GATE").exists(), open,
        "the externally retained keeper must select the declared worker phase");
    let started = selected_path("PSE_ENGINEERING_WORKER_STARTED");
    assert_eq!(started.exists(), open,
        "the original keeper must acknowledge its actual worker launch before completion qualification");
    if open {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Started { version: Version<1> }
        let started: Started = serde_json::from_slice(&std::fs::read(started).unwrap()).unwrap();
        let _version = started.version;
    }
}
fn read_handoff() -> Handoff {
    let handoff: Handoff = serde_json::from_slice(
        &std::fs::read(selected_path("PSE_ENGINEERING_STUDY_HANDOFF")).unwrap()).unwrap();
    assert_eq!(handoff.keeper, keeper_identity(), "frontend restart must retain the same keeper");
    handoff
}
fn key(submitted: &EngineeringSubmitStudy) -> EngineeringStudyKey {
    EngineeringStudyKey { operation: submitted.operation.clone(), study: submitted.study }
}
pub(super) async fn status(client: &EngineeringClient, submitted: &EngineeringSubmitStudy) -> EngineeringStudyStatus {
    codec::decode_engineering_study_status(client.generated().study_status(
        codec::encode_engineering_study_key(key(submitted)).unwrap()).await.unwrap().into_owned()).unwrap()
}
pub(super) async fn submit(client: &EngineeringClient, input: EngineeringSubmitStudy) -> EngineeringStudyStatus {
    codec::decode_engineering_study_status(client.generated().submit_study(
        codec::encode_engineering_submit_study(input).unwrap()).await.unwrap().into_owned()).unwrap()
}
fn observation(submitted: &EngineeringSubmitStudy,
    cursor: Option<pse_runtime::workflow::StudyObservationCursor>) -> EngineeringObserveStudy {
    EngineeringObserveStudy { original: key(submitted),
        request: pse_runtime::workflow::StudyObservationRequest {
            study: submitted.study.to_string(), maximum_members: 2, maximum_events: 1 }, cursor }
}
fn point(case: DeclarationId, occurrence: u32) -> StudyPoint {
    StudyPoint {
        operation: OperationRequest::DeclaredCase(CaseOperation {
            case, route: ModelingAnalysisRoute::Steady,
            settings: pse_runtime::math::settings::SolveSettings {
                intent: pse_backend_native::solve::SolveIntent::Root,
                backend: Some(pse_backend_native::solve::Backend::Kinsol),
                ..Default::default()
            },
        }),
        preparation: PreparationSettings::default(), overlay: PointOverlay::default(),
        policy: StudyPointPolicy { key: OccurrenceKey(occurrence), dependencies: Vec::new(),
            start: StartPolicy::Fresh, attempt_limit: 1 },
    }
}
pub(super) async fn freeze(client: &EngineeringClient, operation: &str, study: &EngineeringSubmitStudy)
    -> pse_runtime::workflow::StudyCohortReceipt {
    let input = EngineeringFreezeStudyCohort { operation: operation.into(),
        request: pse_runtime::workflow::StudyCohortRequest {
            filter: pse_runtime::workflow::StudyWorkbenchFilter {
                study: Some(study.study.to_string()), ..Default::default() },
            maximum_studies: 1, maximum_members: 2,
        } };
    codec::decode_study_cohort_receipt(client.generated().freeze_study_cohort(
        codec::encode_engineering_freeze_study_cohort(input).unwrap()).await.unwrap().into_owned()).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn prepared_engineering_study_client_three_phase_original_owner_journey() {
    let phase = std::env::var("PSE_ENGINEERING_STUDY_PHASE")
        .expect("explicit fixture orchestrator must select submit, recover or complete");
    match phase.as_str() {
        "submit" => phase_submit().await,
        "recover" => phase_recover().await,
        "complete" => phase_complete().await,
        other => panic!("unknown original engineering study phase {other}"),
    }
}

async fn phase_submit() {
    gate_is(false);
    let endpoint = endpoint();
    let frontend = endpoint.invocation.clone();
    let mut client = EngineeringClient::connect(endpoint.clone()).await.unwrap();
    let prefix = format!("study-client-{}", std::process::id());
    let source_operation = format!("{prefix}-source");
    let mut input = source(source_operation.clone());
    input.modeling[0].files.insert("models/root.pse".into(), square(9));
    input.modeling[0].files.insert("models/failure.pse".into(),
        b"package scientific_failure { case impossible { var x:Scalar; eq root:x*x == -1; annotation start x(1); annotation engineering_scale x(kind=magnitude,value=1); } }".to_vec());
    client.generated().admit_source(input).await.unwrap();
    let selected = selected(&mut client, &source_operation).await;
    let original = pse_model::engineering::EngineeringSourceSelection {
        operation: selected.operation, local_revision: selected.local_revision,
        source_revision: ContentHash::from_bytes(selected.source_revision.try_into().unwrap()),
        physical_context: ContentHash::from_bytes(selected.physical_context.try_into().unwrap()),
    };
    let declarations = client.generated().inspect_source(pb::EngineeringInspectSource {
        operation: source_operation.clone(), offset: 0, limit: 128, ..Default::default()
    }).await.unwrap().into_owned();
    let declared = |name| DeclarationId::from_id(SemanticId::from_bytes(
        declarations.declarations.iter().find(|row| row.name == name).unwrap()
            .declaration.as_slice().try_into().unwrap()));
    let prepare = EngineeringPrepareStudy {
        operation: format!("{prefix}-preparation"), source_operation,
        expected_source: original.local_revision.clone(),
        request: StudyRequest { version: Version, output: StudyOutputDemand::Full {},
            inputs: StudyInputs::Explicit(vec![point(declared("square"), 7), point(declared("impossible"), 9)]) },
    };
    let mut stale = prepare.clone();
    stale.expected_source.push_str("-stale");
    stale.operation.push_str("-stale");
    assert!(client.generated().prepare_study(codec::encode_engineering_prepare_study(stale).unwrap()).await.is_err());
    let prepared = codec::decode_engineering_prepared_study(client.generated().prepare_study(
        codec::encode_engineering_prepare_study(prepare.clone()).unwrap()).await.unwrap().into_owned()).unwrap();
    assert_eq!(prepared.occurrences, 2);
    assert_eq!(prepared.source.as_ref(), Some(&original));
    let repeated = codec::decode_engineering_prepared_study(client.generated().prepare_study(
        codec::encode_engineering_prepare_study(prepare).unwrap()).await.unwrap().into_owned()).unwrap();
    assert_eq!(repeated.preparation, prepared.preparation);
    let definition = codec::decode_study_definition(client.generated().read_prepared_study(
        pb::EngineeringOperationKey { operation: prepared.operation.clone(), ..Default::default() })
        .await.unwrap().into_owned()).unwrap();
    assert_eq!(definition.points.len(), 2);
    assert_eq!(pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &definition).unwrap(), prepared.preparation);
    let submitted = EngineeringSubmitStudy { operation: format!("{prefix}-original"),
        preparation_operation: prepared.operation.clone(), expected_preparation: prepared.preparation,
        study: pse_operations::mint_id() };
    let envelope = codec::encode_engineering_submit_study(submitted.clone()).unwrap();
    let lost_ack = tokio::spawn(async move { client.generated().submit_study(envelope).await });
    let observer = EngineeringClient::connect(endpoint).await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(90);
    let queued = loop {
        if let Ok(response) = observer.generated().study_status(
            codec::encode_engineering_study_key(key(&submitted)).unwrap()).await {
            let response = codec::decode_engineering_study_status(response.into_owned()).unwrap();
            assert!(response.failure.is_none(), "original creation refused: {:?}", response.failure);
            if let Some(header) = response.canonical && header.active { break header; }
        }
        assert!(Instant::now() < deadline, "original issued creation was never acknowledged by its owner");
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    lost_ack.abort();
    let _discarded_ack = lost_ack.await;
    assert!(!queued.cancelled && !queued.terminal);
    assert_eq!(queued.point_count, 2);
    assert_eq!(queued.revision, definition.modeling_revision);
    assert_eq!(queued.run, format!("run:{}", pse_runtime::workflow::IssuedStudyIdentity::parent_run_for(submitted.study)));
    let mut stream = observer.generated().observe_study(
        codec::encode_engineering_observe_study(observation(&submitted, None)).unwrap()).await.unwrap();
    let page = codec::decode_study_observation_page(stream.message::<pb::StudyObservationPage>()
        .await.unwrap().unwrap().to_owned_message()).unwrap();
    assert_eq!(page.status.header.study, submitted.study.to_string());
    assert!(!page.status.header.cancelled && !page.status.header.terminal);
    assert!(page.status.members.members.iter().all(|row| !row.value.facts.native_started));
    drop(stream);
    let retired_capture = format!("{prefix}-retired-capture");
    let retired_receipt = freeze(&observer, &retired_capture, &submitted).await;
    assert!(retired_receipt.complete);
    assert_eq!((retired_receipt.header_count, retired_receipt.member_count), (1, 2));
    assert!(observer.generated().read_study_result(codec::encode_engineering_read_study_result(
        EngineeringReadStudyResult { operation: retired_capture.clone(), position: 0,
            relation: "runtime.solve_variables".into(), start: 0, end: 1, maximum_bytes: 1 << 20 },
    ).unwrap()).await.is_err(), "an unattempted capture member has no invented scientific product");
    let cancelled = EngineeringSubmitStudy { operation: format!("{prefix}-cancel"),
        study: pse_operations::mint_id(), ..submitted.clone() };
    let before_cancel = submit(&observer, cancelled.clone()).await.canonical.unwrap();
    assert!(!before_cancel.terminal && !before_cancel.cancelled);
    let cancelled_queued = codec::decode_engineering_study_status(observer.generated().cancel_study(
        codec::encode_engineering_study_key(key(&cancelled)).unwrap()).await.unwrap().into_owned())
        .unwrap().canonical.unwrap();
    assert!(cancelled_queued.cancelled, "cancellation request must reach the original persisted owner");
    let handoff = Handoff { version: Version, keeper: keeper_identity(), frontend, source: original,
        submitted, cancelled, definition, queued, cancelled_queued, retired_capture, retired_receipt,
        cursor: page.cursor };
    let path = selected_path("PSE_ENGINEERING_STUDY_HANDOFF");
    let mut output = tempfile::NamedTempFile::new_in(path.parent().unwrap()).unwrap();
    serde_json::to_writer(output.as_file_mut(), &handoff).unwrap();
    output.as_file().sync_all().unwrap();
    output.persist_noclobber(path).unwrap();
    gate_is(false);
}

async fn phase_recover() {
    gate_is(false);
    let handoff = read_handoff();
    let endpoint = endpoint();
    assert_ne!(endpoint.invocation, handoff.frontend, "recovery must use another authenticated frontend incarnation");
    let client = EngineeringClient::connect(endpoint).await.unwrap();
    let original = submit(&client, handoff.submitted.clone()).await;
    assert!(original.failure.is_none());
    assert_eq!(original.canonical.as_ref(), Some(&handoff.queued));
    assert!(!original.canonical.as_ref().unwrap().cancelled);
    assert!(handoff.retired_receipt.complete);
    assert_eq!((handoff.retired_receipt.header_count, handoff.retired_receipt.member_count), (1, 2));
    assert_eq!(handoff.retired_receipt.request.filter.study.as_deref(), Some(handoff.submitted.study.to_string().as_str()));
    assert_eq!(status(&client, &handoff.cancelled).await.canonical.as_ref(), Some(&handoff.cancelled_queued));
    // Names are local retention only. A new frontend may bind the same preparation name
    // to different science, but the old issued envelope must never adopt that replacement.
    let mut changed = handoff.definition.clone();
    changed.points[0].policy.attempt_limit += 1;
    let changed_identity = pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &changed).unwrap();
    assert_ne!(changed_identity, handoff.submitted.expected_preparation);
    client.generated().restore_study(codec::encode_engineering_restore_study(EngineeringRestoreStudy {
        operation: handoff.submitted.preparation_operation.clone(), expected_preparation: changed_identity,
        definition: changed,
    }).unwrap()).await.unwrap();
    assert!(client.generated().submit_study(codec::encode_engineering_submit_study(EngineeringSubmitStudy {
        expected_preparation: changed_identity, ..handoff.submitted.clone()
    }).unwrap()).await.is_err());
    assert_eq!(submit(&client, handoff.submitted.clone()).await.canonical.as_ref(), Some(&handoff.queued));
    let mut pending = client.generated().observe_study(codec::encode_engineering_observe_study(
        observation(&handoff.submitted, Some(handoff.cursor.clone()))).unwrap()).await.unwrap();
    let pending_page = codec::decode_study_observation_page(pending.message::<pb::StudyObservationPage>()
        .await.unwrap().unwrap().to_owned_message()).unwrap();
    assert_eq!(pending_page.status.header.source, handoff.definition.modeling_revision);
    assert!(!pending_page.status.header.cancelled && !pending_page.status.header.terminal);
    drop(pending);
    let retired = EngineeringStudyHeaderPage { operation: handoff.retired_capture,
        request: pse_runtime::workflow::StudyCohortHeaderRequest { maximum: 1, cursor: None } };
    assert!(client.generated().read_study_headers(codec::encode_engineering_study_header_page(retired).unwrap())
        .await.is_err(), "a portable old receipt must not create a live capture capability in the new frontend");
    gate_is(false);
}

async fn phase_complete() {
    gate_is(true);
    let handoff = read_handoff();
    let client = EngineeringClient::connect(endpoint()).await.unwrap();
    let mut stream = client.generated().observe_study(codec::encode_engineering_observe_study(
        observation(&handoff.submitted, Some(handoff.cursor.clone()))).unwrap()).await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut pages = 0usize;
    let mut last = None;
    loop {
        let remaining = deadline.checked_duration_since(Instant::now()).expect("original study stream deadline");
        let next = tokio::time::timeout(remaining, stream.message::<pb::StudyObservationPage>()).await.unwrap().unwrap();
        let Some(page) = next else { break; };
        let page = codec::decode_study_observation_page(page.to_owned_message()).unwrap();
        assert_eq!(page.status.header.study, handoff.submitted.study.to_string());
        assert_eq!(page.status.header.source, handoff.definition.modeling_revision);
        assert_eq!(page.status.header.physical_identity.as_deref(), Some(handoff.definition.physical.identity.to_string().as_str()));
        assert!(!page.status.header.cancelled);
        assert!(page.sources.iter().map(|source| source.events.len()).sum::<usize>() <= 1);
        for source in &page.sources {
            assert!(source.history_refusal.is_none(), "original terminal history is unavailable: {:?}", source.history_refusal);
            assert!(source.events.len() <= 1, "the original selected event allowance is applied by the producer");
        }
        pages += 1;
        last = Some(page);
    }
    let last = last.expect("original stream must publish at least one native observation");
    assert!(last.status.header.terminal);
    assert!(last.sources.iter().all(|source| source.position.history_complete));
    assert!(pages > 1, "terminal status must not truncate the original one-event history pages");
    drop(stream);
    drop(client);
    let client = EngineeringClient::connect(endpoint()).await.unwrap();
    // Start a fresh bounded history reader after terminal settlement. Even an already
    // terminal first header must retain every unread original history page.
    let mut history = client.generated().observe_study(codec::encode_engineering_observe_study(
        observation(&handoff.submitted, None)).unwrap()).await.unwrap();
    let history_deadline = Instant::now() + Duration::from_secs(90);
    let mut terminal_pages = 0usize;
    let mut incomplete_terminal_prefix = false;
    while let Some(page) = tokio::time::timeout(history_deadline.checked_duration_since(Instant::now())
        .expect("original terminal history deadline"), history.message::<pb::StudyObservationPage>())
        .await.unwrap().unwrap() {
        let page = codec::decode_study_observation_page(page.to_owned_message()).unwrap();
        assert!(page.status.header.terminal);
        assert!(page.sources.iter().map(|source| source.events.len()).sum::<usize>() <= 1);
        incomplete_terminal_prefix |= page.sources.iter().any(|source| !source.position.history_complete);
        terminal_pages += 1;
    }
    assert!(incomplete_terminal_prefix && terminal_pages > 1,
        "the completed numerical cases must exercise an unread terminal-history prefix");
    drop(history);
    let mut foreign_cursor = last.cursor.clone();
    foreign_cursor.context.source.push_str("-foreign");
    let refusal = client.generated().observe_study(codec::encode_engineering_observe_study(
        observation(&handoff.submitted, Some(foreign_cursor))).unwrap()).await;
    if let Ok(mut refusal) = refusal {
        assert!(refusal.message::<pb::StudyObservationPage>().await.is_err(),
            "a foreign original-source cursor must refuse before an observation is published");
    }
    let mut resumed = client.generated().observe_study(codec::encode_engineering_observe_study(
        observation(&handoff.submitted, Some(last.cursor))).unwrap()).await.unwrap();
    let resumed_page = codec::decode_study_observation_page(resumed.message::<pb::StudyObservationPage>()
        .await.unwrap().unwrap().to_owned_message()).unwrap();
    assert!(resumed_page.status.header.terminal);
    assert_eq!(resumed_page.status.header.run, handoff.queued.run);
    assert!(resumed_page.sources.iter().all(|source| source.position.history_complete));
    assert!(resumed.message::<pb::StudyObservationPage>().await.unwrap().is_none());
    let capture_operation = format!("{}-qualified-capture", handoff.submitted.operation);
    let receipt = freeze(&client, &capture_operation, &handoff.submitted).await;
    assert_eq!((receipt.header_count, receipt.member_count), (1, 2));
    assert!(receipt.complete);
    let headers = codec::decode_study_cohort_header_page(client.generated().read_study_headers(
        codec::encode_engineering_study_header_page(EngineeringStudyHeaderPage {
            operation: capture_operation.clone(), request: pse_runtime::workflow::StudyCohortHeaderRequest { maximum: 1, cursor: None },
        }).unwrap()).await.unwrap().into_owned()).unwrap();
    assert_eq!(headers.headers.len(), 1);
    assert!(headers.next.is_none());
    assert_eq!(headers.headers[0].value.source, handoff.definition.modeling_revision);
    assert_eq!(headers.headers[0].value.physical_identity.as_deref(), Some(handoff.definition.physical.identity.to_string().as_str()));
    assert_eq!(headers.headers[0].value.run, handoff.queued.run);
    let mut members = Vec::new();
    let mut cursor = None;
    loop {
        let page = codec::decode_study_cohort_member_page(client.generated().read_study_members(
            codec::encode_engineering_study_cohort_page(EngineeringStudyCohortPage {
                operation: capture_operation.clone(), request: pse_runtime::workflow::StudyCohortPageRequest { maximum: 1, cursor },
            }).unwrap()).await.unwrap().into_owned()).unwrap();
        assert_eq!(page.members.len(), 1);
        members.extend(page.members);
        cursor = page.next;
        if cursor.is_none() { break; }
    }
    assert_eq!(members.len(), 2, "failed and usable siblings both belong to the complete cut");
    let usable = members.iter().find(|member| member.value.facts.scientific.usable).unwrap();
    let failed = members.iter().find(|member| !member.value.facts.scientific.usable).unwrap();
    assert!(usable.value.settled && failed.value.settled);
    assert_eq!(usable.value.facts.key, OccurrenceKey(7));
    assert_eq!(failed.value.facts.key, OccurrenceKey(9));
    assert_eq!(usable.value.study, handoff.submitted.study.to_string());
    let selected = |position, relation: &str, end| EngineeringReadStudyResult {
        operation: capture_operation.clone(), position, relation: relation.into(), start: 0, end,
        maximum_bytes: 1 << 20,
    };
    // Original diagnostic relations can remain readable after scientific failure.
    // Their bytes never change the captured operation's unusable classification.
    match client.generated().read_study_result(codec::encode_engineering_read_study_result(
        selected(usize::try_from(failed.position).unwrap(), "runtime.solve_variables", 16)).unwrap()).await {
        Ok(diagnostic) => {
            let diagnostic = codec::decode_engineering_study_result_page(diagnostic.into_owned()).unwrap();
            assert_eq!(diagnostic.run, failed.value.run);
            assert_eq!(Some(&diagnostic.attempt), failed.value.attempt.as_ref());
            assert!(!failed.value.facts.scientific.usable);
        }
        Err(error) => {
            let failure = super::routes::failure(error);
            assert_eq!(failure.code, pse_diagnostics::DiagnosticCode::WorkflowInput.as_str(),
                "only the original absence/refusal of this diagnostic relation is allowed");
        }
    }
    let variables = read_result(&client, selected(usize::try_from(usable.position).unwrap(), "runtime.solve_variables", 16)).await;
    let numerics = read_result(&client, selected(usize::try_from(usable.position).unwrap(), "runtime.resolved_numerics", 64)).await;
    let mut exhausted_extent = selected(usize::try_from(usable.position).unwrap(), "runtime.solve_variables", 16);
    exhausted_extent.maximum_bytes = 1;
    assert!(client.generated().read_study_result(codec::encode_engineering_read_study_result(exhausted_extent).unwrap())
        .await.is_err(), "a complete IPC stream cannot be replaced by a successful partial response");
    let empty = read_result(&client, EngineeringReadStudyResult {
        start: 0, end: 0, ..selected(usize::try_from(usable.position).unwrap(), "runtime.solve_variables", 0)
    }).await;
    assert_eq!(empty.rows, 0);
    let mut empty_reader = datafusion::arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(empty.ipc), None).unwrap();
    assert!(empty_reader.next().is_none());
    for product in [&variables, &numerics] {
        assert_eq!(product.capture, receipt.capture);
        assert_eq!(product.run, usable.value.run);
        assert_eq!(Some(&product.attempt), usable.value.attempt.as_ref());
        assert_eq!(Some(&product.manifest), usable.value.manifest.as_ref());
    }
    assert_square_original_allowance(&variables, &numerics);
    let release = codec::decode_engineering_cohort_release(client.generated().release_study_cohort(
        pb::EngineeringOperationKey { operation: capture_operation.clone(), ..Default::default() })
        .await.unwrap().into_owned()).unwrap();
    assert!(release.local_admission_closed);
    assert!(client.generated().read_study_result(codec::encode_engineering_read_study_result(
        selected(usize::try_from(usable.position).unwrap(), "runtime.solve_variables", 16)).unwrap()).await.is_err());
    // The already returned immutable bytes remain complete after local native capture release.
    assert_square_original_allowance(&variables, &numerics);
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let cancelled = status(&client, &handoff.cancelled).await.canonical.unwrap();
        assert!(cancelled.cancelled);
        assert_eq!(cancelled.run, handoff.cancelled_queued.run);
        assert_eq!(cancelled.revision, handoff.definition.modeling_revision);
        if cancelled.terminal { break; }
        assert!(Instant::now() < deadline, "accepted cancellation has not established original settlement");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let cancel_capture = format!("{}-settled-capture", handoff.cancelled.operation);
    let cancelled_receipt = freeze(&client, &cancel_capture, &handoff.cancelled).await;
    assert_eq!(cancelled_receipt.member_count, 2);
    let cancelled_members = codec::decode_study_cohort_member_page(client.generated().read_study_members(
        codec::encode_engineering_study_cohort_page(EngineeringStudyCohortPage {
            operation: cancel_capture.clone(), request: pse_runtime::workflow::StudyCohortPageRequest { maximum: 2, cursor: None },
        }).unwrap()).await.unwrap().into_owned()).unwrap();
    assert_eq!(cancelled_members.members.len(), 2);
    assert!(cancelled_members.next.is_none());
    for member in cancelled_members.members {
        assert_eq!(member.value.study, handoff.cancelled.study.to_string());
        assert!(member.value.settled);
        assert!(!member.value.facts.native_started);
        assert!(!member.value.facts.scientific.usable);
    }
    client.generated().release_study_cohort(pb::EngineeringOperationKey {
        operation: cancel_capture, ..Default::default()
    }).await.unwrap();
}

pub(super) async fn read_result(client: &EngineeringClient, input: EngineeringReadStudyResult) -> EngineeringStudyResultPage {
    codec::decode_engineering_study_result_page(client.generated().read_study_result(
        codec::encode_engineering_read_study_result(input).unwrap()).await.unwrap().into_owned()).unwrap()
}

fn assert_square_original_allowance(variables: &EngineeringStudyResultPage, numerics: &EngineeringStudyResultPage) {
    use pse_relations::columnar::ArrowValue as _;
    use pse_model::generated::enums::{EngineeringScaleKind, NumericalSource, NumericalTarget};
    let batches = |ipc: &[u8]| datafusion::arrow::ipc::reader::StreamReader::try_new(
        std::io::Cursor::new(ipc), None).unwrap().map(Result::unwrap).collect::<Vec<_>>();
    let variables_batches = batches(&variables.ipc);
    let policy_batches = batches(&numerics.ipc);
    assert_eq!(variables_batches.iter().map(|batch| batch.num_rows() as u64).sum::<u64>(), variables.rows);
    let mut checked_variables = 0usize;
    let mut checked_parameters = 0usize;
    for batch in &variables_batches {
        for row in 0..batch.num_rows() {
            let field = |name| batch.column_by_name(name).unwrap().as_ref();
            let run = RunId::read(field("run_id"), row).unwrap();
            assert_eq!(format!("run:{run}"), variables.run);
            let value = Option::<f64>::read(field("value"), row).unwrap().expect("original solved value");
            if bool::read(field("parameter"), row).unwrap() {
                assert_eq!(value.to_bits(), 9.0_f64.to_bits());
                checked_parameters += 1;
                continue;
            }
            let symbol = SemanticId::read(field("symbol_id"), row).unwrap();
            let step = i64::read(field("step"), row).unwrap();
            let quantity = Option::<SemanticId>::read(field("quantity_id"), row).unwrap().unwrap();
            let unit = Option::<SemanticId>::read(field("unit_id"), row).unwrap().unwrap();
            let tolerance = Option::<f64>::read(field("tolerance"), row).unwrap().unwrap();
            let mut coordinate_budget = None;
            for policy in &policy_batches {
                for index in 0..policy.num_rows() {
                    let item = |name| policy.column_by_name(name).unwrap().as_ref();
                    assert_eq!(RunId::read(item("run_id"), index).unwrap(), run);
                    if SemanticId::read(item("target_id"), index).unwrap() != symbol
                        || i64::read(item("step"), index).unwrap() != step
                        || NumericalTarget::read(item("target_kind"), index).unwrap() != NumericalTarget::Variable { continue; }
                    assert!(coordinate_budget.is_none());
                    assert_eq!(SemanticId::read(item("quantity_id"), index).unwrap(), quantity);
                    assert_eq!(SemanticId::read(item("unit_id"), index).unwrap(), unit);
                    let engineering = Option::<pse_model::numerics::EngineeringContext>::read(item("engineering"), index)
                        .unwrap().expect("original authored characteristic supplies output-error units and magnitude");
                    assert!(!engineering.canonical_fallback);
                    assert_eq!(engineering.source, NumericalSource::Case);
                    assert_eq!(engineering.scale_kind, Some(EngineeringScaleKind::Magnitude));
                    assert!(engineering.scale_id.is_some());
                    let characteristic = engineering.characteristic.unwrap();
                    assert_eq!(characteristic.to_bits(), 9.0_f64.sqrt().to_bits());
                    assert_eq!(engineering.relative_fraction.to_bits(), pse_model::numerics::NumericalPolicy::default()
                        .engineering_relative_fraction.to_bits());
                    let budget = f64::read(item("budget"), index).unwrap();
                    assert!(budget.is_finite() && budget > 0.0);
                    assert_eq!(budget.to_bits(), (engineering.relative_fraction * characteristic).to_bits());
                    assert_eq!(f64::read(item("absolute"), index).unwrap().to_bits(), budget.to_bits());
                    assert_eq!(f64::read(item("relative"), index).unwrap().to_bits(), 0.0_f64.to_bits());
                    assert_eq!(tolerance.to_bits(), budget.to_bits());
                    coordinate_budget = Some(budget);
                }
            }
            let budget = coordinate_budget.expect("same original variable needs one frozen physical numerical target");
            assert!(value.is_finite() && value > 0.0);
            // Exact positive-root identity in the original variable's quantity/unit;
            // neither native convergence nor a residual alone certifies output error.
            assert!((value * value - 9.0).abs() / (value + 9.0_f64.sqrt()) <= budget);
            assert!((value * value - 4.0).abs() / (value + 4.0_f64.sqrt()) > budget);
            checked_variables += 1;
        }
    }
    assert_eq!((checked_variables, checked_parameters), (1, 1), "both original source members must be read");
}
