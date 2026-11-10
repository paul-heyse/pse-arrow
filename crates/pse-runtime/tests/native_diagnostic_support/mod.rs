// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact accounting and restricted real enclosures through authenticated native RPC.
//! External frontend image/keeper qualification remains the separate H scope.
use super::*;
use pse_ids::SemanticId;
use pse_math::mathematical_diagnostics::{
    DiagnosticClaim, DiagnosticInput, DiagnosticLimits, DiagnosticRequest, ExactValue,
};
use pse_runtime::{
    engineering_projection as codec,
    host_operations::{
        engineering::{
            EngineeringService, LocalAccessPolicy,
            generated::{pb::pse::engineering::v1 as pb, services},
            serve,
        },
        engineering_contract::*,
    },
    workflow::{MathematicalDiagnosticOperation, MathematicalRequest, ModelingInspectionRequest},
};

type Client = services::EngineeringServiceClient<connectrpc::client::SharedHttp2Connection>;
const SOURCE: &str = "package diagnostic_wire {case Root {var x:Scalar;var y:Scalar;var p:Scalar;var q:Scalar;eq f:x+y==1;eq g:x+p*y==q;let h:Scalar=x*x+2*x;let fraction:Scalar=0.1*x;let forbidden:Scalar=sin(x)-sin(x)+x;eq fix_p:p==2;eq fix_q:q==3;annotation start x(0);annotation start y(1);annotation start p(2);annotation start q(3);}}";

async fn connect(socket: &Path) -> Client {
    let uri: http::Uri = "http://localhost".parse().unwrap();
    let connection =
        connectrpc::client::Http2Connection::connect_unix(socket.to_path_buf(), uri.clone())
            .await
            .unwrap()
            .shared(8);
    Client::new(
        connection,
        connectrpc::client::ClientConfig::new(uri)
            .with_protocol(connectrpc::Protocol::Grpc)
            .with_default_max_message_size(16 << 20),
    )
}
fn source(runtime: &Runtime, operation: &str, law: &str) -> pb::EngineeringAdmitSource {
    let directory = root().join("tests/fixtures/packages");
    let physical = pse_admission::authoring_driver::document::capture_package(
        &directory.join("physical-primitives"),
        runtime.registry(),
        Default::default(),
    )
    .unwrap()
    .files;
    let mut manifest: toml::Value = toml::from_str(
        &std::fs::read_to_string(directory.join("minimal_explicit/package.toml")).unwrap(),
    )
    .unwrap();
    manifest["package"]["id_policy"] = "named".into();
    manifest["package"]["dependencies"] = toml::Value::Array(vec![
        toml::Table::from_iter([
            (
                "package_id".into(),
                "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a".into(),
            ),
            (
                "version_req".into(),
                toml::Table::from_iter([
                    ("operator".into(), "exact".into()),
                    ("major".into(), 1.into()),
                    ("minor".into(), 0.into()),
                    ("patch".into(), 0.into()),
                ])
                .into(),
            ),
        ])
        .into(),
    ]);
    let modeling = BTreeMap::from([
        (
            "package.toml".into(),
            toml::to_string(&manifest).unwrap().into_bytes(),
        ),
        ("models/root.pse".into(), law.as_bytes().to_vec()),
    ]);
    pb::EngineeringAdmitSource {
        operation: operation.into(),
        physical: vec![pb::EngineeringPackage {
            files: physical.into_iter().collect(),
            ..Default::default()
        }],
        modeling: vec![pb::EngineeringPackage {
            files: modeling.into_iter().collect(),
            ..Default::default()
        }],
        ..Default::default()
    }
}
async fn admit(
    client: &Client,
    source: pb::EngineeringAdmitSource,
) -> pb::EngineeringSourceSelection {
    let operation = source.operation.clone();
    client.admit_source(source).await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let outcome = client
            .source_status(pb::EngineeringOperationKey {
                operation: operation.clone(),
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
                panic!("original diagnostic source refusal: {value:?}")
            }
            pb::engineering_source_outcome::Kind::Pending(_) => {}
        }
        assert!(Instant::now() < deadline, "source admission deadline");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
async fn settled(
    client: &Client,
    envelope: &EngineeringExecuteMathematical,
) -> pse_model::engineering::EngineeringRunStatus {
    let key = pse_model::engineering::EngineeringRunKey {
        operation: envelope.operation.clone(),
        run: envelope.run,
        attempt: envelope.attempt,
    };
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let status = codec::decode_engineering_run_status(
            client
                .run_status(codec::encode_engineering_run_key(key.clone()).unwrap())
                .await
                .unwrap()
                .into_owned(),
        )
        .unwrap();
        if status.settled {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "mathematical settlement deadline: {status:?}"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
async fn read(
    client: &Client,
    envelope: &EngineeringExecuteMathematical,
) -> pse_model::engineering::EngineeringResultPage {
    codec::decode_engineering_result_page(
        client
            .read_result(
                codec::encode_engineering_read_result(
                    pse_model::engineering::EngineeringReadResult {
                        operation: envelope.operation.clone(),
                        run: envelope.run,
                        attempt: envelope.attempt,
                        relation: "runtime.mathematical_products".into(),
                        start: 0,
                        end: 1,
                        maximum_bytes: 1 << 20,
                    },
                )
                .unwrap(),
            )
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap()
}
fn observation(page: &pse_model::engineering::EngineeringResultPage) -> serde_json::Value {
    use datafusion::arrow::{array::StringArray, ipc::reader::StreamReader};
    let batches = StreamReader::try_new(std::io::Cursor::new(&page.ipc), None)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(page.rows, 1);
    let array = batches[0]
        .column_by_name("result_context")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    serde_json::from_str(array.value(0)).unwrap()
}
async fn exercise(client: &Client, science: &Science) -> (EngineeringExecuteMathematical, Vec<u8>) {
    let runtime = &science.runtime;
    let selected = admit(
        client,
        source(runtime, "diagnostic-original-source", SOURCE),
    )
    .await;
    let declarations = client
        .inspect_source(pb::EngineeringInspectSource {
            operation: "diagnostic-original-source".into(),
            offset: 0,
            limit: 128,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let root = DeclarationId::from_id(SemanticId::from_bytes(
        declarations
            .declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration
            .as_slice()
            .try_into()
            .unwrap(),
    ));
    let inspection = codec::decode_modeling_inspection(
        client
            .inspect_modeling(
                codec::encode_engineering_inspect_modeling(EngineeringInspectModeling {
                    source_operation: "diagnostic-original-source".into(),
                    request: ModelingInspectionRequest {
                        case: root,
                        preparation: PreparationSettings::default(),
                        settings: SolveSettings {
                            intent: SolveIntent::Root,
                            backend: Some(Backend::Ipopt),
                            presolve: PolicyKind::Off,
                            ..Default::default()
                        },
                        offset: 0,
                        limit: 128,
                    },
                })
                .unwrap(),
            )
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    let member = |name: &str| {
        inspection
            .members
            .iter()
            .find(|member| member.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    // The existing public source reader discovers scalar members. This fixture
    // obtains actual original equation IDs from the same first-party authored
    // compiler; execution, settlement and protected reading remain real RPCs.
    let (original, original_root) = package(
        science,
        super::root().join("tests/fixtures/packages/physical-primitives"),
        SOURCE.into(),
    )
    .await;
    assert_eq!(root, original_root);
    let fixture_bindings = pse_modeling::Bindings {
        demand: ["x", "y", "p", "q", "h", "fraction", "forbidden"]
            .map(str::to_owned)
            .to_vec(),
        ..Default::default()
    };
    let compiled = original
        .prepare(
            &runtime.preparation_services(),
            root,
            inspection.selected_instance,
            fixture_bindings.clone(),
            Default::default(),
            &CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(compiled.compiled().model.paths["x"], member("x"));
    let row = |name: &str| {
        compiled
            .compiled()
            .model
            .equations
            .iter()
            .find(|row| row.lineage.path.ends_with(&format!(".{name}")))
            .map(|row| row.id)
            .unwrap_or_else(|| {
                pse_compiler::workspace::ModelingOutput::Member(member(name)).row_id()
            })
    };
    let x = member("x");
    let y = member("y");
    let p = member("p");
    let q = member("q");
    let point = |coordinate, value| DiagnosticInput {
        coordinate,
        lower: ExactValue::integer(value),
        upper: None,
    };
    let operation = MathematicalDiagnosticOperation {
        root,
        instance: inspection.selected_instance,
        bindings: fixture_bindings,
        case_members: BTreeMap::new(),
        request: DiagnosticRequest {
            rows: vec![row("f"), row("g")],
            unknowns: vec![x, y],
            parameters: vec![p, q],
            inputs: vec![point(p, 2), point(q, 3)],
            derivatives: vec![],
            claim: DiagnosticClaim::ExactIdentity,
            precision: 160,
            limits: DiagnosticLimits::default(),
        },
    };
    let envelope = EngineeringExecuteMathematical {
        operation: "wire-diagnostic-exact".into(),
        run: SemanticId::from_bytes([181; 16]),
        attempt: SemanticId::from_bytes([182; 16]),
        source_operation: "diagnostic-original-source".into(),
        expected_local_revision: selected.local_revision,
        request: MathematicalRequest::Diagnostic(operation.clone()),
        preparation: PreparationSettings::default(),
    };
    client
        .execute_mathematical(
            codec::encode_engineering_execute_mathematical(envelope.clone()).unwrap(),
        )
        .await
        .unwrap();
    let status = settled(client, &envelope).await;
    assert!(status.failure.is_none());
    assert_eq!(status.usable, Some(false));
    let page = read(client, &envelope).await;
    let document = observation(&page);
    assert_eq!(document["exact"]["rank"], 2);
    assert_eq!(document["exact"]["consistency"], "unique");
    assert_eq!(document["exact"]["recovered"][0][1], "-1");
    assert_eq!(document["exact"]["recovered"][1][1], "2");
    assert_eq!(document["exact"]["original_domain_valid"], true);
    let mut ball = operation;
    ball.request.rows = vec![row("h")];
    ball.request.unknowns = vec![x];
    ball.request.parameters.clear();
    ball.request.inputs = vec![DiagnosticInput {
        coordinate: x,
        lower: ExactValue::integer(-2),
        upper: Some(ExactValue::integer(3)),
    }];
    ball.request.derivatives = vec![vec![x], vec![x, x]];
    ball.request.claim = DiagnosticClaim::ValidatedEnclosure;
    let mut ball_envelope = envelope.clone();
    ball_envelope.operation = "wire-diagnostic-ball".into();
    ball_envelope.run = SemanticId::from_bytes([183; 16]);
    ball_envelope.attempt = SemanticId::from_bytes([184; 16]);
    ball_envelope.request = MathematicalRequest::Diagnostic(ball.clone());
    client
        .execute_mathematical(
            codec::encode_engineering_execute_mathematical(ball_envelope.clone()).unwrap(),
        )
        .await
        .unwrap();
    assert!(settled(client, &ball_envelope).await.failure.is_none());
    let enclosure = observation(&read(client, &ball_envelope).await);
    assert_eq!(enclosure["claim"], "validated_enclosure");
    assert_eq!(enclosure["numerical"][1]["lower"], "-2");
    assert_eq!(enclosure["numerical"][1]["upper"], "8");
    assert_eq!(enclosure["numerical"][2]["lower"], "2");
    assert!(enclosure["numerical"][0]["value"].is_null());
    let mut fraction = ball.clone();
    fraction.request.rows = vec![row("fraction")];
    fraction.request.inputs = vec![point(x, 1)];
    fraction.request.derivatives.clear();
    let mut fractional = ball_envelope.clone();
    fractional.operation = "wire-diagnostic-fraction".into();
    fractional.run = SemanticId::from_bytes([188; 16]);
    fractional.attempt = SemanticId::from_bytes([189; 16]);
    fractional.request = MathematicalRequest::Diagnostic(fraction);
    client
        .execute_mathematical(
            codec::encode_engineering_execute_mathematical(fractional.clone()).unwrap(),
        )
        .await
        .unwrap();
    assert!(settled(client, &fractional).await.failure.is_none());
    let fractional = observation(&read(client, &fractional).await);
    assert_eq!(
        fractional["numerical"][0]["lower"],
        "3602879701896397/36028797018963968"
    );
    assert_eq!(
        fractional["numerical"][0]["lower"],
        fractional["numerical"][0]["upper"]
    );
    // Source-level transcendental obligations survive cancellation in supplier atoms.
    ball.request.rows = vec![row("forbidden")];
    ball.request.derivatives.clear();
    let mut refused = ball_envelope;
    refused.operation = "wire-diagnostic-forbidden".into();
    refused.run = SemanticId::from_bytes([185; 16]);
    refused.attempt = SemanticId::from_bytes([186; 16]);
    refused.request = MathematicalRequest::Diagnostic(ball);
    client
        .execute_mathematical(
            codec::encode_engineering_execute_mathematical(refused.clone()).unwrap(),
        )
        .await
        .unwrap();
    assert!(settled(client, &refused).await.failure.is_some());
    let changed = admit(
        client,
        source(
            runtime,
            "diagnostic-changed-source",
            &SOURCE.replace("2*x", "3*x"),
        ),
    )
    .await;
    let mut rebound = envelope.clone();
    rebound.source_operation = "diagnostic-changed-source".into();
    rebound.expected_local_revision = changed.local_revision;
    assert!(
        client
            .execute_mathematical(codec::encode_engineering_execute_mathematical(rebound).unwrap())
            .await
            .is_err()
    );
    assert_eq!(read(client, &envelope).await.ipc, page.ipc);
    (envelope, page.ipc)
}

async fn journey(science: &Science) {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("diagnostics.sock");
    let reader = Science::new(
        science
            .runtime
            .canonical_store()
            .unwrap()
            .independent_caller()
            .unwrap(),
        true,
    );
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    let service = EngineeringService::new(
        science.runtime.clone(),
        science.shared.clone(),
        16,
        BTreeMap::new(),
    )
    .unwrap();
    let stop = CancelSource::new();
    let server = tokio::spawn(serve(
        listener,
        service,
        LocalAccessPolicy {
            uid: rustix::process::geteuid().as_raw(),
            gid: None,
            maximum_connections: 8,
            connection_drain: Duration::from_secs(10),
        },
        stop.clone(),
    ));
    let body = AssertUnwindSafe(async {
        let client = connect(&socket).await;
        exercise(&client, science).await
    })
    .catch_unwind()
    .await;
    stop.cancel();
    let drained = server.await.unwrap();
    let (envelope, bytes) = match body {
        Ok(value) => {
            drained.unwrap();
            value
        }
        Err(panic) => {
            let closed = reader.close().await;
            eprintln!("diagnostic RPC producer drain={drained:?};reader={closed:?}");
            std::panic::resume_unwind(panic)
        }
    };
    let socket = directory.path().join("reader.sock");
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    let service = EngineeringService::new(
        reader.runtime.clone(),
        reader.shared.clone(),
        8,
        BTreeMap::new(),
    )
    .unwrap();
    let stop = CancelSource::new();
    let server = tokio::spawn(serve(
        listener,
        service,
        LocalAccessPolicy {
            uid: rustix::process::geteuid().as_raw(),
            gid: None,
            maximum_connections: 8,
            connection_drain: Duration::from_secs(10),
        },
        stop.clone(),
    ));
    let body = AssertUnwindSafe(async {
        let client = connect(&socket).await;
        let repeated = codec::decode_engineering_run_status(
            client
                .execute_mathematical(
                    codec::encode_engineering_execute_mathematical(envelope.clone()).unwrap(),
                )
                .await
                .unwrap()
                .into_owned(),
        )
        .unwrap();
        assert!(repeated.settled);
        assert_eq!(repeated.usable, Some(false));
        assert_eq!(read(&client, &envelope).await.ipc, bytes);
        let mut wrong = envelope;
        wrong.attempt = SemanticId::from_bytes([187; 16]);
        assert!(
            client
                .read_result(
                    codec::encode_engineering_read_result(
                        pse_model::engineering::EngineeringReadResult {
                            operation: wrong.operation,
                            run: wrong.run,
                            attempt: wrong.attempt,
                            relation: "runtime.mathematical_products".into(),
                            start: 0,
                            end: 1,
                            maximum_bytes: 1 << 20
                        }
                    )
                    .unwrap()
                )
                .await
                .is_err()
        );
    })
    .catch_unwind()
    .await;
    stop.cancel();
    let drained = server.await.unwrap();
    let closed = reader.close().await;
    if let Err(panic) = body {
        eprintln!("diagnostic RPC reader drain={drained:?};native={closed:?}");
        std::panic::resume_unwind(panic)
    }
    drained.unwrap();
    closed.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn public_native_diagnostic_rpc_preserves_exact_claim_enclosure_and_original_reader() {
    let mut service = service();
    let outcome = AssertUnwindSafe(service.run(
        Instant::now() + Duration::from_secs(600),
        |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let science = Science::new(service.ready()?.store().clone(), false);
                let body = AssertUnwindSafe(journey(&science)).catch_unwind().await;
                let closed = science.close().await;
                if let Err(panic) = body {
                    eprintln!(
                        "diagnostic native producer drain={closed:?};state={}",
                        service.state().display()
                    );
                    std::panic::resume_unwind(panic)
                }
                closed.unwrap();
                Ok(())
            })
        },
    ))
    .catch_unwind()
    .await;
    match outcome {
        Ok(Ok(())) => service.dispose().unwrap(),
        Ok(Err(error)) => panic!(
            "diagnostic native failure={error};state={}",
            service.state().display()
        ),
        Err(panic) => {
            eprintln!(
                "diagnostic native retained state={}",
                service.state().display()
            );
            std::panic::resume_unwind(panic)
        }
    }
}
