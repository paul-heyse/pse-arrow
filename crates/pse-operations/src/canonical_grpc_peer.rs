// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Real HTTP/2/protobuf peer for canonical native transport controls.

use base64::Engine;
use bytes::Bytes;
use http_body::Frame;
use http_body_util::{BodyExt, StreamBody};
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::{convert::Infallible, sync::Arc};
use surrealdb_protocol::proto::{rpc::v1 as rpc, v1 as proto};
use surrealdb_types::prost::Message;
use tokio::sync::{Semaphore, mpsc};

pub(super) struct Peer {
    pub address: std::net::SocketAddr,
    pub received: mpsc::UnboundedReceiver<String>,
    pub methods: mpsc::UnboundedReceiver<String>,
    #[cfg(feature = "canonical-tests")]
    pub requests: mpsc::UnboundedReceiver<(std::time::Instant, rpc::QueryRequest)>,
    pub release: Arc<Semaphore>,
    pub server: tokio::task::JoinHandle<()>,
}

fn frame(message: impl Message) -> Vec<u8> {
    let bytes = message.encode_to_vec();
    let mut framed = vec![0];
    framed.extend_from_slice(&u32::try_from(bytes.len()).unwrap().to_be_bytes());
    framed.extend(bytes);
    framed
}
fn unsupported_statement_frames(live: bool) -> Vec<u8> {
    let mut bytes = frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::Begin(rpc::QueryBegin {
            query_id: Some(proto::Uuid::from_uuid(uuid::Uuid::new_v4())),
            statement_count: 1,
        })),
    });
    for (batch_index, kind, values) in if live {
        vec![(
            0,
            rpc::QueryResponseKind::Single,
            vec!["unsupported-live-id"],
        )]
    } else {
        vec![
            (0, rpc::QueryResponseKind::Batched, vec!["first", "second"]),
            (1, rpc::QueryResponseKind::Single, vec!["last"]),
        ]
    } {
        bytes.extend(frame(rpc::QueryResponse {
            frame: Some(rpc::query_response::Frame::Batch(rpc::QueryBatchFrame {
                query_index: 0,
                batch_index,
                kind: kind as i32,
                statement_kind: if live {
                    rpc::QueryStatementKind::Live
                } else {
                    rpc::QueryStatementKind::Other
                } as i32,
                payload: Some(rpc::query_batch_frame::Payload::Values(rpc::ValueBatch {
                    values: values
                        .into_iter()
                        .map(|value| proto::Value {
                            value: Some(if live {
                                proto::value::Value::Uuid(proto::Uuid::from_uuid(
                                    uuid::Uuid::new_v4(),
                                ))
                            } else {
                                proto::value::Value::String(value.into())
                            }),
                        })
                        .collect(),
                })),
                ..Default::default()
            })),
        }));
    }
    bytes.extend(frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::End(rpc::QueryEnd {
            result_count: 1,
            ..Default::default()
        })),
    }));
    bytes
}
fn sparse_kill_frames() -> Vec<u8> {
    let mut bytes = frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::Begin(rpc::QueryBegin {
            query_id: Some(proto::Uuid::from_uuid(uuid::Uuid::new_v4())),
            statement_count: 5,
        })),
    });
    for (index, kind, text) in [
        (1, rpc::QueryStatementKind::Other, "first"),
        (3, rpc::QueryStatementKind::Kill, "kill"),
        (4, rpc::QueryStatementKind::Other, "last"),
    ] {
        bytes.extend(frame(rpc::QueryResponse {
            frame: Some(rpc::query_response::Frame::Batch(rpc::QueryBatchFrame {
                query_index: index,
                statement_kind: kind as i32,
                kind: rpc::QueryResponseKind::Single as i32,
                payload: Some(rpc::query_batch_frame::Payload::Values(rpc::ValueBatch {
                    values: vec![proto::Value {
                        value: Some(proto::value::Value::String(text.into())),
                    }],
                })),
                ..Default::default()
            })),
        }));
    }
    bytes.extend(frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::End(rpc::QueryEnd {
            result_count: 3,
            ..Default::default()
        })),
    }));
    bytes
}
fn query_frames(value: proto::Value, complete: bool) -> Vec<u8> {
    let mut bytes = frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::Begin(rpc::QueryBegin {
            query_id: Some(proto::Uuid::from_uuid(uuid::Uuid::new_v4())),
            statement_count: 1,
        })),
    });
    bytes.extend(frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::Batch(rpc::QueryBatchFrame {
            query_index: 0,
            kind: rpc::QueryResponseKind::Single as i32,
            payload: Some(rpc::query_batch_frame::Payload::Values(rpc::ValueBatch {
                values: vec![value],
            })),
            ..Default::default()
        })),
    }));
    if complete {
        bytes.extend(frame(rpc::QueryResponse {
            frame: Some(rpc::query_response::Frame::End(rpc::QueryEnd {
                result_count: 1,
                ..Default::default()
            })),
        }));
    }
    bytes
}
fn multiple_error_frames() -> Vec<u8> {
    let mut bytes = frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::Begin(rpc::QueryBegin {
            query_id: Some(proto::Uuid::from_uuid(uuid::Uuid::new_v4())),
            statement_count: 3,
        })),
    });
    // Arrival order differs from statement order; cancellation precedes both
    // substantive failures in the statement inventory.
    for (index, message, details) in [
        (2, "second substantive", "TransactionConflict"),
        (0, "not executed", "NotExecuted"),
        (1, "first substantive", "TransactionConflict"),
    ] {
        bytes.extend(frame(rpc::QueryResponse {
            frame: Some(rpc::query_response::Frame::Batch(rpc::QueryBatchFrame {
                query_index: index,
                kind: rpc::QueryResponseKind::Single as i32,
                error: Some(
                    proto::SurrealError::new(proto::ErrorKind::Query, message)
                        .with_details(details, None),
                ),
                ..Default::default()
            })),
        }));
    }
    bytes.extend(frame(rpc::QueryResponse {
        frame: Some(rpc::query_response::Frame::End(rpc::QueryEnd {
            result_count: 3,
            ..Default::default()
        })),
    }));
    bytes
}

fn generated_block_page() -> proto::Value {
    let rows = (0..64_u64)
        .map(|ordinal| {
            let key = format!("block-{ordinal}");
            let row = pse_model::generated::runtime::canonical_result_blocks::Row {
                key: key.clone(),
                result_set: "result-set".into(),
                batch: key.clone(),
                output: "temperature".into(),
                partition: "time".into(),
                ordinal,
                start: ordinal,
                end: ordinal + 1,
                rows: 1,
                columns: 1,
                coordinate_min: Some(ordinal as f64),
                coordinate_max: Some(ordinal as f64),
                payload_bytes: 2,
                payload_digest: "payload-digest".into(),
                statistics: None,
                interpretation: crate::generated::surreal::INTERPRETATION.into(),
            };
            let mut object =
                crate::generated::surreal::encode_canonical_result_blocks(&row).unwrap();
            object.insert(
                "id",
                crate::canonical_codec::record_link("canonical_result_blocks", &key),
            );
            surrealdb_types::Value::Object(object)
        })
        .collect::<Vec<_>>();
    proto::Value::try_from(surrealdb_types::Value::Array(rows.into())).unwrap()
}

pub(super) async fn start() -> Peer {
    start_generations(1).await
}
pub(super) async fn start_generations(generations: usize) -> Peer {
    start_with_restore_delay(generations, false).await
}
pub(super) async fn start_with_restore_delay(generations: usize, delay: bool) -> Peer {
    start_scripted(generations, delay, None, false).await
}
#[cfg(feature = "canonical-tests")]
pub(super) async fn start_source_stage_fault(pending_readback: bool) -> Peer {
    start_scripted(1, false, Some(pending_readback), false).await
}
pub(super) async fn start_with_held_detach() -> Peer {
    start_scripted(1, false, None, true).await
}
async fn start_scripted(
    generations: usize,
    delay: bool,
    source_stage_fault: Option<bool>,
    hold_detach: bool,
) -> Peer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (sent, received) = mpsc::unbounded_channel();
    let (methods_sent, methods) = mpsc::unbounded_channel();
    #[cfg(feature = "canonical-tests")]
    let (requests_sent, requests) = mpsc::unbounded_channel();
    let release = Arc::new(Semaphore::new(0));
    let peer_release = release.clone();
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let read_conflicts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let receipt_reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = tokio::spawn(async move {
        for generation in 0..generations {
            let (socket, _) = listener.accept().await.unwrap();
            let sent = sent.clone();
            let methods_sent = methods_sent.clone();
            #[cfg(feature = "canonical-tests")]
            let requests_sent = requests_sent.clone();
            let peer_release = peer_release.clone();
            let barrier = barrier.clone();
            let read_conflicts = read_conflicts.clone();
            let receipt_reads = receipt_reads.clone();
            let service = hyper::service::service_fn(
                move |request: http::Request<hyper::body::Incoming>| {
                    let methods_sent = methods_sent.clone();
                    #[cfg(feature = "canonical-tests")]
                    let requests_sent = requests_sent.clone();
                    let sent = sent.clone();
                    let release = peer_release.clone();
                    let barrier = barrier.clone();
                    let read_conflicts = read_conflicts.clone();
                    let receipt_reads = receipt_reads.clone();
                    async move {
                        let method = request.uri().path().rsplit('/').next().unwrap().to_owned();
                        let _ = methods_sent.send(method.clone());
                        let body = request.into_body().collect().await.unwrap().to_bytes();
                        let payload = body.get(5..).unwrap_or_default();
                        let mut status = "0";
                        let mut native_error = None;
                        let bytes = match method.as_str() {
                            "GetCapabilities" => frame(rpc::GetCapabilitiesResponse {
                                capabilities: Some(rpc::ServerCapabilities {
                                    server_version: "3.3.4".into(),
                                    ..Default::default()
                                }),
                            }),
                            "AttachSession" => frame(rpc::AttachSessionResponse {
                                session: Some(proto::Uuid::from_uuid(uuid::Uuid::new_v4())),
                                ..Default::default()
                            }),
                            "Signin" => {
                                let signin = rpc::SigninRequest::decode(payload).unwrap();
                                if delay && generation == 1 {
                                    release.acquire().await.unwrap().forget();
                                }
                                let rejected = matches!(signin.access_method.and_then(|a| a.method), Some(rpc::access_method::Method::User(user)) if user.username == "rejected");
                                if rejected {
                                    status = "7";
                                    Vec::new()
                                } else {
                                    frame(rpc::SigninResponse {
                                        tokens: Some(rpc::Tokens {
                                            access: "test-only-token".into(),
                                            ..Default::default()
                                        }),
                                    })
                                }
                            }
                            "Use" => frame(rpc::UseResponse {
                                namespace: "namespace".into(),
                                database: "database".into(),
                            }),
                            "Query" => {
                                let request = rpc::QueryRequest::decode(payload).unwrap();
                                #[cfg(feature = "canonical-tests")]
                                let _ = requests_sent
                                    .send((std::time::Instant::now(), request.clone()));
                                let canonical_probe=request.variables.as_ref().is_some_and(|variables|variables.variables.iter().any(|entry|
                                    entry.key=="pse_record" && matches!(entry.value.as_ref().and_then(|value|value.value.as_ref()),Some(proto::value::Value::RecordId(record)) if record.table=="canonical_interpretations")
                                ));
                                let query = request.query;
                                let _ = sent.send(query.clone());
                                if query.contains("FIRST_READ") || query.contains("SECOND_READ") {
                                    barrier.wait().await;
                                }
                                if query.contains("HOLD") {
                                    release.acquire().await.unwrap().forget();
                                }
                                let source_receipt = source_stage_fault.is_some()
                                    && query.contains("SELECT * FROM ONLY type::record('canonical_revisions', $key)");
                                let source_readback = source_receipt
                                    && receipt_reads
                                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                                        > 0;
                                let source_begin = source_stage_fault.is_some()
                                    && query.contains("LET $stage_guard =");
                                if source_readback && source_stage_fault == Some(true) {
                                    release.acquire().await.unwrap().forget();
                                }
                                if source_begin || source_readback {
                                    if source_begin {
                                        status = "10";
                                        native_error = Some(
                                            proto::SurrealError::new(
                                                proto::ErrorKind::Query,
                                                "source stage statement refusal",
                                            )
                                            .with_details("Cancelled", None)
                                            .encode_to_vec(),
                                        );
                                    } else {
                                        status = "14";
                                        native_error = Some(
                                            proto::SurrealError::new(
                                                proto::ErrorKind::Connection,
                                                "source receipt connection reply",
                                            )
                                            .encode_to_vec(),
                                        );
                                    }
                                    Vec::new()
                                } else if query.contains("MIXED_BATCH_TO_SINGLE") {
                                    unsupported_statement_frames(false)
                                } else if query.contains("UNSUPPORTED_LIVE_RESPONSE") {
                                    unsupported_statement_frames(true)
                                } else if query.contains("SPARSE_WITH_KILL") {
                                    sparse_kill_frames()
                                } else if query.contains("MULTIPLE_STATEMENT_ERRORS") {
                                    multiple_error_frames()
                                } else if query.contains("READ_ALWAYS_CONFLICT")
                                    || (query.contains("READ_CONFLICT_TWICE")
                                        && read_conflicts
                                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                                            < 2)
                                    || query.contains("READ_CANCELLED")
                                {
                                    status = "10";
                                    native_error = Some(
                                        proto::SurrealError::new(
                                            proto::ErrorKind::Query,
                                            "scripted complete read refusal",
                                        )
                                        .with_details(
                                            if query.contains("READ_CANCELLED") {
                                                "Cancelled"
                                            } else {
                                                "TransactionConflict"
                                            },
                                            None,
                                        )
                                        .encode_to_vec(),
                                    );
                                    Vec::new()
                                } else if query.contains("MALFORMED") {
                                    vec![0, 0, 0, 0, 2, 0x3a, 255]
                                } else {
                                    let value = if query.contains("GENERATED_BLOCK_PAGE") {
                                        generated_block_page()
                                    } else if query.contains("BLOB_FOUR") {
                                        proto::Value {
                                            value: Some(proto::value::Value::Bytes(
                                                vec![1u8; 4].into(),
                                            )),
                                        }
                                    } else if query.contains("BLOB_FIVE") {
                                        proto::Value {
                                            value: Some(proto::value::Value::Bytes(
                                                vec![1u8; 5].into(),
                                            )),
                                        }
                                    } else if query.contains("BIG_STRING") {
                                        proto::Value {
                                            value: Some(proto::value::Value::String(
                                                "x".repeat(8192),
                                            )),
                                        }
                                    } else if source_receipt
                                        || (source_stage_fault.is_some()
                                            && query.contains("canonical_project_operations"))
                                        || query.contains("canonical_interpretations")
                                        || canonical_probe
                                    {
                                        proto::Value {
                                            value: Some(proto::value::Value::None(
                                                proto::NoneValue {},
                                            )),
                                        }
                                    } else {
                                        proto::Value {
                                            value: Some(proto::value::Value::Bool(true)),
                                        }
                                    };
                                    let mut bytes = query_frames(value, !query.contains("NO_END"));
                                    if query.contains("AGGREGATE") {
                                        bytes = frame(rpc::QueryResponse {
                                            frame: Some(rpc::query_response::Frame::Begin(
                                                rpc::QueryBegin {
                                                    query_id: Some(proto::Uuid::from_uuid(
                                                        uuid::Uuid::new_v4(),
                                                    )),
                                                    statement_count: 1,
                                                },
                                            )),
                                        });
                                        for index in 0..3 {
                                            bytes.extend(frame(rpc::QueryResponse { frame: Some(rpc::query_response::Frame::Batch(rpc::QueryBatchFrame { query_index: 0, batch_index: index, kind: if index == 2 { rpc::QueryResponseKind::BatchedFinal } else { rpc::QueryResponseKind::Batched } as i32, payload: Some(rpc::query_batch_frame::Payload::Values(rpc::ValueBatch { values: vec![proto::Value { value: Some(proto::value::Value::String("x".repeat(4096))) }] })), ..Default::default() })) }));
                                        }
                                        bytes.extend(frame(rpc::QueryResponse {
                                            frame: Some(rpc::query_response::Frame::End(
                                                rpc::QueryEnd {
                                                    result_count: 1,
                                                    ..Default::default()
                                                },
                                            )),
                                        }));
                                    }
                                    if query.contains("COUNT_BOMB") {
                                        bytes = frame(rpc::QueryResponse {
                                            frame: Some(rpc::query_response::Frame::Begin(
                                                rpc::QueryBegin {
                                                    query_id: Some(proto::Uuid::from_uuid(
                                                        uuid::Uuid::new_v4(),
                                                    )),
                                                    statement_count: 1 << 20,
                                                },
                                            )),
                                        });
                                    }
                                    if query.contains("SPARSE_INDEX") {
                                        bytes = frame(rpc::QueryResponse {
                                            frame: Some(rpc::query_response::Frame::Begin(
                                                rpc::QueryBegin {
                                                    query_id: Some(proto::Uuid::from_uuid(
                                                        uuid::Uuid::new_v4(),
                                                    )),
                                                    statement_count: 1,
                                                },
                                            )),
                                        });
                                        bytes.extend(frame(rpc::QueryResponse {
                                            frame: Some(rpc::query_response::Frame::Batch(
                                                rpc::QueryBatchFrame {
                                                    query_index: (1 << 20) - 1,
                                                    kind: rpc::QueryResponseKind::Single as i32,
                                                    ..Default::default()
                                                },
                                            )),
                                        }));
                                    }
                                    if query.contains("SKIPPED_VALID") {
                                        bytes = frame(rpc::QueryResponse {
                                            frame: Some(rpc::query_response::Frame::Begin(
                                                rpc::QueryBegin {
                                                    query_id: Some(proto::Uuid::from_uuid(
                                                        uuid::Uuid::new_v4(),
                                                    )),
                                                    statement_count: 3,
                                                },
                                            )),
                                        });
                                        bytes.extend(frame(rpc::QueryResponse {
                                            frame: Some(rpc::query_response::Frame::Batch(
                                                rpc::QueryBatchFrame {
                                                    query_index: 1,
                                                    kind: rpc::QueryResponseKind::Single as i32,
                                                    payload: Some(
                                                        rpc::query_batch_frame::Payload::Values(
                                                            rpc::ValueBatch {
                                                                values: vec![proto::Value {
                                                                    value: Some(
                                                                        proto::value::Value::Bool(
                                                                            true,
                                                                        ),
                                                                    ),
                                                                }],
                                                            },
                                                        ),
                                                    ),
                                                    ..Default::default()
                                                },
                                            )),
                                        }));
                                        bytes.extend(frame(rpc::QueryResponse {
                                            frame: Some(rpc::query_response::Frame::End(
                                                rpc::QueryEnd {
                                                    result_count: 1,
                                                    ..Default::default()
                                                },
                                            )),
                                        }));
                                    }
                                    if query.contains("REPEATED_BEGIN") {
                                        bytes = query_frames(
                                            proto::Value {
                                                value: Some(proto::value::Value::Bool(true)),
                                            },
                                            false,
                                        );
                                        bytes.extend(query_frames(
                                            proto::Value {
                                                value: Some(proto::value::Value::Bool(true)),
                                            },
                                            true,
                                        ));
                                    }
                                    if query.contains("LATE_FAILURE") {
                                        bytes = query_frames(
                                            proto::Value {
                                                value: Some(proto::value::Value::Bool(true)),
                                            },
                                            false,
                                        );
                                        bytes.extend(frame(rpc::QueryResponse {
                                            frame: Some(rpc::query_response::Frame::Error(
                                                proto::SurrealError {
                                                    kind: proto::ErrorKind::Internal as i32,
                                                    message: "test late transaction failure".into(),
                                                    ..Default::default()
                                                },
                                            )),
                                        }));
                                    }
                                    bytes
                                }
                            }
                            "BeginTransaction" => frame(rpc::BeginTransactionResponse {
                                transaction: Some(proto::Uuid::from_uuid(uuid::Uuid::new_v4())),
                            }),
                            "CommitTransaction" => {
                                status = "10";
                                native_error = Some(
                                    proto::SurrealError::new(
                                        proto::ErrorKind::Query,
                                        "opaque reason",
                                    )
                                    .with_details("TransactionConflict", None)
                                    .encode_to_vec(),
                                );
                                Vec::new()
                            }
                            "DetachSession" => {
                                if hold_detach {
                                    release.acquire().await.unwrap().forget();
                                }
                                frame(rpc::DetachSessionResponse {})
                            }
                            "SetVariable" | "UnsetVariable" | "Health" => {
                                frame(rpc::HealthResponse {})
                            }
                            _ => {
                                status = "12";
                                Vec::new()
                            }
                        };
                        let mut trailers = http::HeaderMap::new();
                        trailers.insert("grpc-status", status.parse().unwrap());
                        if let Some(error) = native_error {
                            trailers.insert(
                                "surreal-error-bin",
                                base64::engine::general_purpose::STANDARD_NO_PAD
                                    .encode(error)
                                    .parse()
                                    .unwrap(),
                            );
                        }
                        let frames = vec![
                            Ok::<_, Infallible>(Frame::data(Bytes::from(bytes))),
                            Ok(Frame::trailers(trailers)),
                        ];
                        let mut response = http::Response::new(StreamBody::new(
                            futures_util::stream::iter(frames),
                        ));
                        response
                            .headers_mut()
                            .insert("content-type", "application/grpc".parse().unwrap());
                        Ok::<_, Infallible>(response)
                    }
                },
            );
            let _ = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(socket), service)
                .await;
        }
    });
    Peer {
        address,
        received,
        methods,
        #[cfg(feature = "canonical-tests")]
        requests,
        release,
        server,
    }
}
