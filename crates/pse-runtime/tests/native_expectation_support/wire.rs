// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Production generated DTOs/handlers over kernel-authenticated Unix HTTP/2.
//! The external frontend image/keeper remains the assembled deployment qualification's scope.
use super::*;
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
    workflow::{
        ModelingInspectionRequest, PhysicalInspectionDocument, PhysicalInspectionQuery,
        PhysicalInspectionRequest,
    },
};

type Client = services::EngineeringServiceClient<connectrpc::client::SharedHttp2Connection>;
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

fn source(runtime: &Runtime) -> pb::EngineeringAdmitSource {
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
    let modeling=BTreeMap::from([("package.toml".into(),toml::to_string(&manifest).unwrap().into_bytes()),("models/root.pse".into(),b"package wire_expectation {case Population {param x:Scalar=0;var y:Scalar;eq response:y==x;annotation start y(0);}}".to_vec())]);
    pb::EngineeringAdmitSource {
        operation: "wire-expectation-source".into(),
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

async fn exercise(
    client: &Client,
    runtime: &Runtime,
) -> (
    StudyId,
    ExpectationReport,
    Vec<pse_model::generated::runtime::expectation_samples::Row>,
) {
    client.admit_source(source(runtime)).await.unwrap();
    loop {
        let status = client
            .source_status(pb::EngineeringOperationKey {
                operation: "wire-expectation-source".into(),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_owned();
        match status.kind.unwrap() {
            pb::engineering_source_outcome::Kind::Ready(_) => break,
            pb::engineering_source_outcome::Kind::Failed(error) => {
                panic!("original wire source failed: {error:?}")
            }
            pb::engineering_source_outcome::Kind::Pending(_) => {
                tokio::time::sleep(Duration::from_millis(5)).await
            }
        }
    }
    let declarations = client
        .inspect_source(pb::EngineeringInspectSource {
            operation: "wire-expectation-source".into(),
            offset: 0,
            limit: 128,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let root = pse_model::generated::identities::DeclarationId::from_id(SemanticId::from_bytes(
        declarations
            .declarations
            .iter()
            .find(|declaration| declaration.name == "Population")
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
                    source_operation: "wire-expectation-source".into(),
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
    let input = inspection
        .members
        .iter()
        .find(|member| member.lineage.path.ends_with(".x"))
        .unwrap();
    let physical = codec::decode_physical_inspection_document(
        client
            .inspect_physical(pb::EngineeringInspectPhysical {
                source_operation: "wire-expectation-source".into(),
                request: codec::encode_physical_inspection_request(PhysicalInspectionRequest {
                    query: PhysicalInspectionQuery::QuantityNamed {
                        name: "Scalar".into(),
                    },
                    offset: 0,
                    limit: 1,
                })
                .unwrap()
                .into(),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    let PhysicalInspectionDocument::Page { quantities, .. } = physical else {
        panic!("original physical Scalar required")
    };
    let scalar = &quantities[0];
    let value = |magnitude| BindingQuantity {
        quantity: scalar.quantity.id,
        unit: scalar.canonical_unit.id,
        magnitude: FiniteBound::try_new(magnitude).unwrap(),
    };
    let parent = StudyId::from_id(SemanticId::from_bytes([55; 16]));
    let request = ExpectationRequest {
        source: inspection.source,
        preparation: PreparationSettings::default(),
        member: ExpectationMember::Observation(SelectedObservationOperation {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: pse_modeling::Bindings {
                demand: vec!["x".into()],
                ..Default::default()
            },
            outputs: vec![SelectedObservationOutput {
                member: BindingTarget::Member(input.id),
                quantity: scalar.quantity.id,
                unit: scalar.canonical_unit.id,
            }],
        }),
        coordinates: vec![ExpectationCoordinate::Uniform {
            target: BindingTarget::Member(input.id),
            lower: value(0.),
            upper: value(1.),
        }],
        transform: None,
        outputs: vec![ExpectationOutput {
            output: input.id,
            quantity: scalar.quantity.id,
            unit: scalar.canonical_unit.id,
            reference: None,
            scale: value(1.),
            absolute_error: None,
            relative_error: None,
            train: true,
        }],
        estimator: ExpectationEstimator::FixedMonteCarlo,
        controls: ExpectationControls {
            seed: 17,
            lanes: 2,
            samples_per_iteration: 16,
            maximum_iterations: 1,
            maximum_samples: 16,
            normalization_tolerance: 1e-12,
        },
        failure_probability: true,
        replay: ExpectationReplay::PopulationAndBoundary,
    };
    let envelope = EngineeringExecuteExpectation {
        operation: "wire-expectation-execution".into(),
        source_operation: "wire-expectation-source".into(),
        parent,
        request,
    };
    let encoded = codec::encode_engineering_execute_expectation(envelope.clone()).unwrap();
    let report = codec::decode_expectation_report(
        client
            .execute_expectation(encoded.clone())
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    assert_eq!(report.issued, 16);
    assert!(report.outputs[0].complete);
    let repeated = codec::decode_expectation_report(
        client
            .execute_expectation(encoded)
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_vec(&report).unwrap(),
        serde_json::to_vec(&repeated).unwrap()
    );
    let mut foreign = envelope.clone();
    foreign.operation = "wire-expectation-foreign-source".into();
    foreign.parent = StudyId::from_id(SemanticId::from_bytes([57; 16]));
    foreign.request.source.physical_context = pse_ids::ContentHash::from_bytes([99; 32]);
    assert!(
        client
            .execute_expectation(codec::encode_engineering_execute_expectation(foreign).unwrap())
            .await
            .is_err()
    );
    let mut malformed = envelope;
    malformed.operation = "wire-expectation-malformed".into();
    malformed.parent = StudyId::from_id(SemanticId::from_bytes([56; 16]));
    if let ExpectationCoordinate::Uniform { lower, upper, .. } =
        &mut malformed.request.coordinates[0]
    {
        *upper = lower.clone();
    }
    assert!(
        client
            .execute_expectation(codec::encode_engineering_execute_expectation(malformed).unwrap())
            .await
            .is_err()
    );
    let page = codec::decode_engineering_expectation_samples(
        client
            .read_expectation_samples(
                codec::encode_engineering_read_expectation_samples(
                    EngineeringReadExpectationSamples {
                        parent,
                        iteration: 0,
                        offset: 0,
                        limit: 16,
                    },
                )
                .unwrap(),
            )
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    assert_eq!(page.rows.len(), 16);
    let mut finite_sum=0_f64;
    for row in &page.rows {
        let original=row.base_coordinates[0];
        assert!((row.inputs[0].canonical-original).abs()<1e-14);
        assert!((row.outputs[0].canonical.unwrap()-original).abs()<1e-14,"the reconstructed selected parameter remains the physical output, not its case residual");
        finite_sum+=original;
    }
    assert!((report.outputs[0].estimate.unwrap()-finite_sum/16.).abs()<1e-14);
    (parent, report, page.rows)
}

pub(super) async fn journey(science: &Science) {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("expectation.sock");
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
        exercise(&client, &science.runtime).await
    })
    .catch_unwind()
    .await;
    stop.cancel();
    let drained = server.await.unwrap();
    let (parent, report, rows) = match body {
        Ok(value) => {
            drained.unwrap();
            value
        }
        Err(panic) => {
            let closed = reader.close().await;
            eprintln!("expectation producer RPC drain={drained:?};reader={closed:?}");
            std::panic::resume_unwind(panic)
        }
    };
    // A new source-free frontend and a new HTTP/2 client read the protected parent
    // after the original frontend has released its package and mathematical services.
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
        let read = codec::decode_expectation_report(
            client
                .read_expectation(
                    codec::encode_engineering_read_expectation(EngineeringReadExpectation {
                        parent,
                    })
                    .unwrap(),
                )
                .await
                .unwrap()
                .into_owned(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_vec(&read).unwrap(),
            serde_json::to_vec(&report).unwrap()
        );
        let page = codec::decode_engineering_expectation_samples(
            client
                .read_expectation_samples(
                    codec::encode_engineering_read_expectation_samples(
                        EngineeringReadExpectationSamples {
                            parent,
                            iteration: 0,
                            offset: 0,
                            limit: 16,
                        },
                    )
                    .unwrap(),
                )
                .await
                .unwrap()
                .into_owned(),
        )
        .unwrap();
        assert_eq!(page.rows, rows);
    })
    .catch_unwind()
    .await;
    stop.cancel();
    let drained = server.await.unwrap();
    let closed = reader.close().await;
    if let Err(panic) = body {
        eprintln!("expectation reader RPC drain={drained:?};native={closed:?}");
        std::panic::resume_unwind(panic);
    }
    drained.unwrap();
    closed.unwrap();
}
