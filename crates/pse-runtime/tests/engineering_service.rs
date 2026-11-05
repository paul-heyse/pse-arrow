// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent generated client of an explicitly prepared original managed service.
#![cfg(feature = "engineering-service")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "prepared qualification inputs and exact original-owner assertions"
)]

use pse_runtime::host_operations::engineering::{
    EngineeringClient, EngineeringEndpoint, EngineeringFixtureReady, EngineeringFrontendReady,
    generated::pb::pse::engineering::v1 as pb,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[cfg(all(feature = "solver-diffsol", feature = "solver-ipopt"))]
#[path = "engineering_service/routes.rs"]
mod routes;
#[path = "engineering_service/study.rs"]
mod study;

fn failure(error: connectrpc::ConnectError) -> pb::EngineeringFailure {
    use base64::{
        Engine as _,
        engine::general_purpose::{STANDARD, STANDARD_NO_PAD},
    };
    use buffa::Message as _;
    let detail = error
        .details
        .iter()
        .find(|detail| {
            matches!(
                detail.type_url.as_str(),
                "pse.engineering.v1.EngineeringFailure"
                    | "type.googleapis.com/pse.engineering.v1.EngineeringFailure"
            )
        })
        .expect("retain original typed owner refusal");
    let encoded = detail.value.as_ref().unwrap();
    let bytes = STANDARD_NO_PAD
        .decode(encoded)
        .or_else(|_| STANDARD.decode(encoded))
        .unwrap();
    let failure = pb::EngineeringFailure::decode_from_slice(&bytes).unwrap();
    assert!(!failure.code.is_empty());
    failure
}

fn endpoint() -> EngineeringEndpoint {
    let path = std::env::var_os("PSE_ENGINEERING_ENDPOINT")
        .expect("explicit fixture owner must prepare an engineering payload before this client; reads never launch it");
    let ready: EngineeringFrontendReady =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let _version = ready.version;
    assert!(ready.process > 0 && ready.build != pse_ids::ContentHash::NIL);
    EngineeringEndpoint {
        root: ready.root,
        invocation: ready.invocation,
        socket: ready.socket,
        environment: std::env::vars_os().collect(),
        artifact: ready.artifact,
        build: ready.build,
    }
}
async fn selected(
    client: &mut EngineeringClient,
    operation: &str,
) -> pb::EngineeringSourceSelection {
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let outcome = client
            .generated()
            .source_status(pb::EngineeringOperationKey {
                operation: operation.into(),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_owned();
        match outcome.kind.unwrap() {
            pb::engineering_source_outcome::Kind::Ready(value) => {
                return value.value.into_option().unwrap();
            }
            pb::engineering_source_outcome::Kind::Failed(value) => {
                panic!(
                    "original source operation {operation} refusal: {:?}",
                    value.value
                )
            }
            pb::engineering_source_outcome::Kind::Pending(_) => {}
        }
        assert!(
            Instant::now() < deadline,
            "original source admission did not complete"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
async fn settled(
    client: &mut EngineeringClient,
    key: &pb::EngineeringRunKey,
) -> pb::EngineeringRunStatus {
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let status = client
            .generated()
            .run_status(key.clone())
            .await
            .unwrap()
            .into_owned();
        if status.settled {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "original native run has not established settlement: {status:?}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
fn source(operation: String) -> pb::EngineeringAdmitSource {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let physical = root.join("physical-primitives");
    let physical_files = [
        "package.toml",
        "materials/physical.yaml",
        "materials/time.yaml",
    ]
    .into_iter()
    .map(|path| (path.into(), std::fs::read(physical.join(path)).unwrap()))
    .collect();
    let manifest = std::fs::read_to_string(root.join("minimal_named/package.toml")).unwrap()
        .replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
    let files = BTreeMap::from([
        ("package.toml".into(), manifest.into_bytes()),
        ("models/root.pse".into(), square(4)),
    ]);
    pb::EngineeringAdmitSource {
        operation,
        physical: vec![pb::EngineeringPackage {
            files: physical_files,
            ..Default::default()
        }],
        modeling: vec![pb::EngineeringPackage {
            files: files.into_iter().collect(),
            ..Default::default()
        }],
        ..Default::default()
    }
}
fn square(parameter: i32) -> Vec<u8> {
    format!("package local {{ case square {{ param p:Scalar={parameter}; var x:Scalar; eq root:x*x==p; annotation start x(2); annotation engineering_scale x(kind=magnitude,value=sqrt(p)); }} }}").into_bytes()
}
fn package_id() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/packages/minimal_named/package.toml");
    let manifest: toml::Value = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let id = manifest["package"]["id"].as_str().unwrap().replace('-', "");
    pse_ids::SemanticId::parse_hex(&id)
        .unwrap()
        .as_bytes()
        .to_vec()
}
async fn edit(
    client: &mut EngineeringClient,
    operation: String,
    parent_operation: &str,
    parent: &str,
    path: &str,
    change: pb::source_file_change::Kind,
) -> pb::EngineeringSourceSelection {
    client
        .generated()
        .edit_source(pb::EngineeringEditSource {
            operation: operation.clone(),
            source_operation: parent_operation.into(),
            delta: pb::SourceFileDelta {
                parent: parent.into(),
                edits: vec![pb::SourceFileEdit {
                    package: package_id(),
                    path: path.into(),
                    change: pb::SourceFileChange {
                        kind: Some(change),
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                }],
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .await
        .unwrap();
    selected(client, &operation).await
}

#[tokio::test(flavor = "multi_thread")]
async fn prepared_engineering_connector_refuses_stale_and_replaced_original_payloads() {
    let selected = endpoint();
    let mut wrong = selected.clone();
    wrong.invocation = pse_host::envelope::CliInvocationId::new();
    assert!(EngineeringClient::connect(wrong).await.is_err());
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("replacement.sock");
    let _replacement = tokio::net::UnixListener::bind(&socket).unwrap();
    let mut replaced = selected;
    replaced.socket = socket;
    // This is a real kernel peer from another executable/invocation, even though
    // it has the same UID. Authentication must refuse before HTTP/2 negotiation.
    assert!(EngineeringClient::connect(replaced).await.is_err());
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test(flavor = "multi_thread")]
async fn prepared_engineering_client_preserves_original_edit_run_and_reconnect() {
    use pb::source_file_change::{Alternative1, Alternative2, Kind};
    use sha2::Digest as _;
    let selected_endpoint = endpoint();
    let mut client = EngineeringClient::connect(selected_endpoint.clone())
        .await
        .unwrap();
    let described = client
        .generated()
        .describe_contract(pb::EngineeringDescribeRequest::default())
        .await
        .unwrap()
        .into_owned();
    assert_eq!(
        described.descriptor_sha256,
        sha2::Sha256::digest(&described.descriptor).to_vec()
    );
    assert!(
        described
            .methods
            .iter()
            .any(|method| method.name == "SubmitRun"
                && !method.effects.read_only
                && method.effects.idempotent)
    );
    assert!(
        described
            .methods
            .iter()
            .any(|method| method.name == "ObserveRun"
                && method.server_streaming
                && method.effects.read_only)
    );
    let prefix = format!("independent-{}", std::process::id());
    let original_operation = format!("{prefix}-source");
    let request = source(original_operation.clone());
    client
        .generated()
        .admit_source(request.clone())
        .await
        .unwrap();
    let original = selected(&mut client, &original_operation).await;
    let physical_request = pse_runtime::engineering_projection::encode_physical_inspection_request(
        pse_runtime::workflow::PhysicalInspectionRequest {
            query: pse_runtime::workflow::PhysicalInspectionQuery::QuantityNamed {
                name: "Length".into(),
            },
            offset: 0,
            limit: 1,
        },
    )
    .unwrap();
    let physical = client
        .generated()
        .inspect_physical(pb::EngineeringInspectPhysical {
            source_operation: original_operation.clone(),
            request: physical_request.into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let physical =
        pse_runtime::engineering_projection::decode_physical_inspection_document(physical).unwrap();
    let pse_runtime::workflow::PhysicalInspectionDocument::Page {
        context,
        count,
        quantities,
        ..
    } = physical
    else {
        panic!("actual selected physical context must be inspectable")
    };
    assert_eq!(
        context.as_bytes().as_slice(),
        original.physical_context.as_slice()
    );
    assert_eq!(count, 1);
    assert_eq!(quantities.len(), 1);
    let quantity = &quantities[0];
    assert_eq!(quantity.canonical_unit.symbol, "m");
    assert_eq!(quantity.quantity.key.kind, quantity.quantity_kind.id);
    assert_eq!(quantity.quantity.canonical_unit, quantity.canonical_unit.id);
    assert_eq!(quantity.representation_unit.id, quantity.canonical_unit.id);
    assert_eq!(quantity.scale_to_canonical.to_bits(), 1.0_f64.to_bits());
    assert_eq!(quantity.offset_to_canonical.to_bits(), 0.0_f64.to_bits());
    assert!(quantity.basis.is_none() && quantity.reference_state.is_none());
    let repeat = client
        .generated()
        .admit_source(request.clone())
        .await
        .unwrap()
        .into_owned();
    let pb::engineering_source_outcome::Kind::Ready(repeat) = repeat.kind.unwrap() else {
        panic!("exact repeated source must resolve its original result")
    };
    assert_eq!(repeat.value.local_revision, original.local_revision);
    let mut changed = request;
    changed.modeling[0]
        .files
        .insert("models/root.pse".into(), square(9));
    assert!(client.generated().admit_source(changed).await.is_err());

    let added_operation = format!("{prefix}-add");
    let added = edit(
        &mut client,
        added_operation.clone(),
        &original_operation,
        &original.local_revision,
        "models/retained.pse",
        Kind::Add(Box::new(Alternative1 {
            value: b"package retained { param retained:Scalar=3; }".to_vec(),
            ..Default::default()
        })),
    )
    .await;
    let replaced_operation = format!("{prefix}-replace");
    let replaced = edit(
        &mut client,
        replaced_operation.clone(),
        &added_operation,
        &added.local_revision,
        "models/root.pse",
        Kind::Replace(Box::new(Alternative2 {
            value: square(9),
            ..Default::default()
        })),
    )
    .await;
    let inherited = client
        .generated()
        .inspect_source(pb::EngineeringInspectSource {
            operation: replaced_operation.clone(),
            offset: 0,
            limit: 128,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    assert!(
        inherited
            .declarations
            .iter()
            .any(|declaration| declaration.name == "retained")
    );
    let native_file_request = pse_runtime::workflow::OriginalSourceFileRequest {
        parent: replaced.local_revision.clone(),
        package: pse_ids::SemanticId::from_bytes(package_id().try_into().unwrap()),
        path: "models/root.pse".into(),
        maximum_bytes: 1 << 20,
    };
    let file_request = pse_runtime::engineering_projection::encode_original_source_file_request(
        native_file_request,
    )
    .unwrap();
    let original_file = client
        .generated()
        .read_source_file(pb::EngineeringReadSourceFile {
            source_operation: replaced_operation.clone(),
            request: file_request.into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let original_file =
        pse_runtime::engineering_projection::decode_original_source_file(original_file).unwrap();
    assert_eq!(original_file.local_revision, replaced.local_revision);
    assert_eq!(original_file.path, "models/root.pse");
    assert_eq!(original_file.body, square(9));
    assert_eq!(
        original_file.original_hash,
        pse_ids::encoding_checksum(&original_file.body).content_hash()
    );
    assert!(original_file.is_text && !original_file.document_version.is_empty());
    let changed_body = std::str::from_utf8(&original_file.body)
        .unwrap()
        .replace("p:Scalar=9", "p:Scalar=16")
        .into_bytes();
    assert_ne!(changed_body, original_file.body);
    let inspected_edit_operation = format!("{prefix}-inspected-edit");
    let inspected_edit = edit(
        &mut client,
        inspected_edit_operation.clone(),
        &replaced_operation,
        &original_file.local_revision,
        "models/root.pse",
        Kind::Replace(Box::new(Alternative2 {
            value: changed_body,
            ..Default::default()
        })),
    )
    .await;
    let removed_operation = format!("{prefix}-remove");
    let removed = edit(
        &mut client,
        removed_operation.clone(),
        &inspected_edit_operation,
        &inspected_edit.local_revision,
        "models/retained.pse",
        Kind::Remove(Box::default()),
    )
    .await;
    let inspected = client
        .generated()
        .inspect_source(pb::EngineeringInspectSource {
            operation: removed_operation.clone(),
            offset: 0,
            limit: 128,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    assert!(
        !inspected
            .declarations
            .iter()
            .any(|declaration| declaration.name == "retained")
    );
    let case = inspected
        .declarations
        .iter()
        .find(|declaration| declaration.name == "square")
        .unwrap()
        .declaration
        .clone();
    let discovery_request =
        pse_runtime::engineering_projection::encode_preparation_discovery_request(
            pse_runtime::workflow::PreparationDiscoveryRequest {
                root: pse_model::generated::identities::DeclarationId::from_id(
                    pse_ids::SemanticId::from_bytes(case.clone().try_into().unwrap()),
                ),
                instance: pse_model::generated::identities::InstanceId::from_bytes([71; 16]),
                bindings: Default::default(),
                limits: Default::default(),
                durable_after: None,
            },
        )
        .unwrap();
    let discovered = client
        .generated()
        .discover_preparation(pb::EngineeringDiscoverPreparation {
            source_operation: removed_operation.clone(),
            request: discovery_request.into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let native_discovery =
        pse_runtime::engineering_projection::decode_preparation_discovery_report(discovered)
            .unwrap();
    assert_eq!(native_discovery.revision_id, removed.local_revision);
    assert!(
        native_discovery.resident.len()
            <= pse_runtime::workflow::PreparationDiscoveryReport::MAX_RESIDENT_ROWS
    );
    assert_eq!(
        native_discovery.physical,
        pse_ids::ContentHash::from_bytes(removed.physical_context.clone().try_into().unwrap())
            .to_hex()
    );
    let durable = native_discovery.durable.expect(
        "the explicitly canonical source retains its original rooted publication selection",
    );
    assert!(
        durable.rows.is_empty() && durable.next.is_none(),
        "this exact newly edited source has no prior rooted preparation publication"
    );
    assert_eq!(
        native_discovery.durable_eligibility,
        pse_runtime::workflow::PreparationEligibility::Unchecked,
        "original durable presence grants no receiving scientific qualification"
    );
    let final_file_request =
        pse_runtime::engineering_projection::encode_original_source_file_request(
            pse_runtime::workflow::OriginalSourceFileRequest {
                parent: removed.local_revision.clone(),
                package: pse_ids::SemanticId::from_bytes(package_id().try_into().unwrap()),
                path: "models/root.pse".into(),
                maximum_bytes: 1 << 20,
            },
        )
        .unwrap();
    let final_file = client
        .generated()
        .read_source_file(pb::EngineeringReadSourceFile {
            source_operation: removed_operation.clone(),
            request: final_file_request.into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let final_file =
        pse_runtime::engineering_projection::decode_original_source_file(final_file).unwrap();
    assert_eq!(final_file.local_revision, removed.local_revision);
    assert_eq!(
        final_file.body,
        square(16),
        "the exact source after inspected replacement and unrelated-file removal must retain p=16"
    );
    assert_eq!(
        final_file.original_hash,
        pse_ids::encoding_checksum(&square(16)).content_hash()
    );
    let prepared = client
        .generated()
        .prepare_case(pb::EngineeringPrepareCase {
            operation: format!("{prefix}-prepare"),
            source_operation: removed_operation,
            expected_local_revision: removed.local_revision,
            case,
            intent: pb::NativeSolveIntent {
                value: pse_model::generated::enums::NativeSolveIntent::Root
                    .as_str()
                    .into(),
                ..Default::default()
            }
            .into(),
            backend: pb::NativeBackend {
                value: pse_model::generated::enums::NativeBackend::Kinsol
                    .as_str()
                    .into(),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let run: pse_model::generated::identities::RunId = pse_operations::mint_id();
    let attempt: pse_model::generated::identities::AttemptId = pse_operations::mint_id();
    let submit = pb::EngineeringSubmitRun {
        operation: format!("{prefix}-run"),
        preparation_operation: prepared.operation,
        run: run.as_id().as_bytes().to_vec(),
        attempt: attempt.as_id().as_bytes().to_vec(),
        expected_preparation: prepared.preparation,
        ..Default::default()
    };
    let key = pb::EngineeringRunKey {
        operation: submit.operation.clone(),
        run: submit.run.clone(),
        attempt: submit.attempt.clone(),
        ..Default::default()
    };
    // The application deliberately loses the submit acknowledgement. A separate
    // connection observes the original issued coordinates before that waiter is joined/dropped.
    let submit_copy = submit.clone();
    let acknowledgement =
        tokio::spawn(async move { client.generated().submit_run(submit_copy).await });
    let observer = EngineeringClient::connect(selected_endpoint.clone())
        .await
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        if let Ok(status) = observer.generated().run_status(key.clone()).await
            && status.into_owned().run.is_some()
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "original submitted identity did not become observable"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    acknowledgement.abort();
    let _discarded_acknowledgement = acknowledgement.await;
    let mut progress = observer.generated().observe_run(key.clone()).await.unwrap();
    let observed = progress
        .message::<pb::EngineeringRunStatus>()
        .await
        .unwrap()
        .unwrap();
    drop(observed);
    drop(progress);
    drop(observer);
    let mut reconnected = EngineeringClient::connect(selected_endpoint).await.unwrap();
    let finished = settled(&mut reconnected, &key).await;
    assert_eq!(finished.run.as_deref(), Some(submit.run.as_slice()));
    assert_eq!(finished.usable, Some(true));
    assert!(finished.canonical_run.is_some() && finished.canonical_attempt.is_some());
    let repeated = reconnected
        .generated()
        .submit_run(submit.clone())
        .await
        .unwrap()
        .into_owned();
    assert_eq!(repeated.run, finished.run);
    let output = reconnected
        .generated()
        .read_result(pb::EngineeringReadResult {
            operation: submit.operation,
            run: submit.run,
            attempt: submit.attempt,
            relation: "runtime.solve_variables".into(),
            start: 0,
            end: 1,
            maximum_bytes: 1 << 20,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    assert_eq!(output.rows, 1);
    let mut stream = datafusion::arrow::ipc::reader::StreamReader::try_new(
        std::io::Cursor::new(output.ipc),
        None,
    )
    .unwrap();
    let batch = stream.next().unwrap().unwrap();
    assert_eq!(batch.num_rows(), 1);
    let expected_run =
        pse_model::generated::identities::RunId::from_bytes(key.run.as_slice().try_into().unwrap());
    assert_eq!(
        <pse_model::generated::identities::RunId as pse_relations::columnar::ArrowValue>::read(
            batch.column_by_name("run_id").unwrap().as_ref(),
            0,
        )
        .unwrap(),
        expected_run,
        "the selected variable belongs to the exact original caller-issued run"
    );
    let values = batch
        .column_by_name("value")
        .unwrap()
        .as_any()
        .downcast_ref::<datafusion::arrow::array::Float64Array>()
        .unwrap();
    let parameters = batch
        .column_by_name("parameter")
        .unwrap()
        .as_any()
        .downcast_ref::<datafusion::arrow::array::BooleanArray>()
        .unwrap();
    let parameter = parameters.value(0);
    let expected = if parameter { 16.0 } else { 4.0 };
    let actual = if datafusion::arrow::array::Array::is_null(values, 0) {
        None
    } else {
        Some(values.value(0))
    };
    let tolerances = batch
        .column_by_name("tolerance")
        .unwrap()
        .as_any()
        .downcast_ref::<datafusion::arrow::array::Float64Array>()
        .unwrap();
    let native_tolerance = if datafusion::arrow::array::Array::is_null(tolerances, 0) {
        None
    } else {
        Some(tolerances.value(0))
    };
    let matches_original_source = if parameter {
        actual.map(f64::to_bits) == Some(16.0f64.to_bits())
    } else {
        use pse_model::generated::enums::{EngineeringScaleKind, NumericalSource, NumericalTarget};
        use pse_relations::columnar::ArrowValue as _;
        let symbol =
            pse_ids::SemanticId::read(batch.column_by_name("symbol_id").unwrap().as_ref(), 0)
                .unwrap();
        let step = i64::read(batch.column_by_name("step").unwrap().as_ref(), 0).unwrap();
        let quantity = Option::<pse_ids::SemanticId>::read(
            batch.column_by_name("quantity_id").unwrap().as_ref(),
            0,
        )
        .unwrap();
        let unit = Option::<pse_ids::SemanticId>::read(
            batch.column_by_name("unit_id").unwrap().as_ref(),
            0,
        )
        .unwrap();
        let published = reconnected
            .generated()
            .read_result(pb::EngineeringReadResult {
                operation: key.operation.clone(),
                run: key.run.clone(),
                attempt: key.attempt.clone(),
                relation: "runtime.resolved_numerics".into(),
                start: 0,
                end: 16,
                maximum_bytes: 1 << 20,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_owned();
        let policy = datafusion::arrow::ipc::reader::StreamReader::try_new(
            std::io::Cursor::new(published.ipc),
            None,
        )
        .unwrap();
        let mut coordinate_budget = None;
        for page in policy {
            let page = page.unwrap();
            for row in 0..page.num_rows() {
                let field = |name| page.column_by_name(name).unwrap().as_ref();
                assert_eq!(
                    pse_model::generated::identities::RunId::read(field("run_id"), row).unwrap(),
                    expected_run,
                    "the frozen numerical target belongs to the same exact original run"
                );
                if pse_ids::SemanticId::read(field("target_id"), row).unwrap() != symbol
                    || i64::read(field("step"), row).unwrap() != step
                    || NumericalTarget::read(field("target_kind"), row).unwrap()
                        != NumericalTarget::Variable
                {
                    continue;
                }
                assert!(
                    coordinate_budget.is_none(),
                    "one original variable target must govern this coordinate"
                );
                assert_eq!(
                    Some(pse_ids::SemanticId::read(field("quantity_id"), row).unwrap()),
                    quantity
                );
                assert_eq!(
                    Some(pse_ids::SemanticId::read(field("unit_id"), row).unwrap()),
                    unit
                );
                let nominal = f64::read(field("nominal"), row).unwrap();
                let scale = f64::read(field("coordinate_scale"), row).unwrap();
                let absolute = f64::read(field("absolute"), row).unwrap();
                let relative = f64::read(field("relative"), row).unwrap();
                let budget = f64::read(field("budget"), row).unwrap();
                assert!(nominal.is_finite() && nominal > 0.0 && scale.is_finite() && scale > 0.0);
                assert!(budget.is_finite() && budget > 0.0);
                assert_eq!(budget.to_bits(), (absolute + relative * nominal).to_bits());
                let engineering = Option::<pse_model::numerics::EngineeringContext>::read(
                    field("engineering"),
                    row,
                )
                .unwrap()
                .expect(
                    "the authored characteristic supplies the original engineering interpretation",
                );
                assert!(
                    !engineering.canonical_fallback,
                    "canonical magnitude fallback is not meaningful output-error evidence"
                );
                assert_eq!(engineering.source, NumericalSource::Case);
                assert_eq!(
                    engineering.scale_kind,
                    Some(EngineeringScaleKind::Magnitude)
                );
                assert!(engineering.scale_id.is_some());
                let characteristic = engineering
                    .characteristic
                    .expect("authored input-derived characteristic");
                assert_eq!(characteristic.to_bits(), 16.0f64.sqrt().to_bits());
                assert_eq!(
                    engineering.relative_fraction.to_bits(),
                    pse_model::numerics::NumericalPolicy::default()
                        .engineering_relative_fraction
                        .to_bits()
                );
                assert_eq!(
                    engineering.budget.to_bits(),
                    (engineering.relative_fraction * characteristic).to_bits()
                );
                assert_eq!(absolute.to_bits(), engineering.budget.to_bits());
                assert_eq!(relative.to_bits(), 0.0f64.to_bits());
                assert_eq!(native_tolerance.map(f64::to_bits), Some(budget.to_bits()));
                coordinate_budget = Some(budget);
            }
        }
        let budget = coordinate_budget
            .expect("same original symbol/step must have its frozen numerical target");
        // Independent exact positive-root identity, in x's original quantity/unit.
        // A native convergence or engineering allowance alone does not guarantee this error.
        actual.is_some_and(|value| {
            value.is_finite()
                && value > 0.0
                && (value * value - 16.0).abs() / (value + 16.0f64.sqrt()) <= budget
                && (value * value - 9.0).abs() / (value + 9.0f64.sqrt()) > budget
                && (value * value - 4.0).abs() / (value + 4.0f64.sqrt()) > budget
        })
    };
    assert!(
        matches_original_source,
        "the original edited source must govern the native numerical product: actual={actual:?}, expected={expected}, parameter={parameter}, source_member={}, symbol={:?}, step={:?}, native_tolerance={:?}",
        if parameter { "p" } else { "x" },
        batch.column_by_name("symbol_id"),
        batch.column_by_name("step"),
        batch.column_by_name("tolerance"),
    );
    assert!(stream.next().is_none());
    let cancelled = reconnected
        .generated()
        .cancel_run(key.clone())
        .await
        .unwrap()
        .into_owned();
    assert!(cancelled.cancellation_requested);
    let after_cancel = settled(&mut reconnected, &key).await;
    assert_eq!(after_cancel.run, finished.run);
}
