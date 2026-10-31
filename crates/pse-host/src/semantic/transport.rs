// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Narrow initialization adapter: lspf owns correlation/cancellation; lsp-server owns framing.
use super::{Error, Result};
use bytes::Bytes;
use lspf::types::{InitializeParams, InitializeResult, WorkspaceFolder};
use lspf::{RawMessage, RequestId, Transport, TransportError, TransportReader, TransportWriter};
use std::{
    borrow::Cow,
    io::BufReader,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::{mpsc, oneshot};

/// One native post-activation configuration exchange, owned by lspf's handler lifecycle.
pub(super) struct ConfigurationExchange {
    pub(super) options: Mutex<serde_json::Value>,
    // 0: loading; 1: selected reverse config; 2: false response actually written;
    // 3: native ordered roundtrip completed; 4: original exchange failure retained.
    pub(super) phase: AtomicUsize,
    pub(super) issue: Mutex<Option<String>>,
    pub(super) written: tokio::sync::Notify,
    request: Mutex<Option<RequestId>>,
}
impl ConfigurationExchange {
    pub(super) fn new(options: serde_json::Value) -> Self {
        Self {
            options: Mutex::new(options),
            phase: AtomicUsize::new(0),
            issue: Mutex::new(None),
            written: tokio::sync::Notify::new(),
            request: Mutex::new(None),
        }
    }
}

#[derive(Default)]
pub(super) struct Evidence {
    pub(super) result: Option<InitializeResult>,
    id: Option<RequestId>,
    pub(super) status: Option<serde_json::Value>,
}
pub(super) struct ChildOwner(pub(super) Child);
impl Drop for ChildOwner {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
pub(super) struct NativeTransport {
    reader: Reader,
    writer: Writer,
}
pub(super) struct Reader {
    messages: mpsc::Receiver<std::result::Result<RawMessage, TransportError>>,
}
pub(super) struct Writer {
    messages: std::sync::mpsc::Sender<(
        RawMessage,
        oneshot::Sender<std::result::Result<(), TransportError>>,
    )>,
    evidence: Arc<Mutex<Evidence>>,
    root: String,
    configuration: Arc<ConfigurationExchange>,
}
impl Transport for NativeTransport {
    type Reader = Reader;
    type Writer = Writer;
    fn split(self) -> (Reader, Writer) {
        (self.reader, self.writer)
    }
}
impl TransportReader for Reader {
    async fn recv(&mut self) -> std::result::Result<RawMessage, TransportError> {
        self.messages
            .recv()
            .await
            .unwrap_or(Err(TransportError::Closed))
    }
}
impl TransportWriter for Writer {
    async fn send(&mut self, mut message: RawMessage) -> std::result::Result<(), TransportError> {
        let configuration_response = if let RawMessage::Response {
            id,
            result: Ok(bytes),
        } = &message
        {
            self.configuration
                .request
                .lock()
                .map_err(malformed)?
                .as_ref()
                == Some(id)
                && serde_json::from_slice::<serde_json::Value>(bytes)
                    .ok()
                    .is_some_and(|value| value[0]["cargo"]["autoreload"] == false)
        } else {
            false
        };
        if let RawMessage::Request { id, method, params } = &mut message
            && method == "initialize"
        {
            let mut request: InitializeParams =
                serde_json::from_slice(params).map_err(malformed)?;
            request.process_id = Some(std::process::id().try_into().map_err(malformed)?);
            request
                .workspace_folders_initialize_params
                .workspace_folders = Some(
                vec![WorkspaceFolder {
                    uri: self.root.parse().map_err(malformed)?,
                    name: "selected-project".into(),
                }]
                .into(),
            );
            *params = Bytes::from(serde_json::to_vec(&request).map_err(malformed)?);
            self.evidence.lock().map_err(malformed)?.id = Some(id.clone());
        }
        let (sent, completed) = oneshot::channel();
        self.messages
            .send((message, sent))
            .map_err(|_| TransportError::Closed)?;
        let result = completed.await.map_err(|_| TransportError::Closed)?;
        if result.is_ok()
            && configuration_response
            && self
                .configuration
                .phase
                .compare_exchange(1, 2, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        {
            self.configuration.written.notify_one();
        }
        result
    }
    async fn shutdown(self) -> std::result::Result<(), TransportError> {
        drop(self.messages);
        Ok(())
    }
}
fn malformed(error: impl std::fmt::Display) -> TransportError {
    TransportError::Malformed(error.to_string())
}
fn native_id(id: RequestId) -> std::result::Result<lsp_server::RequestId, TransportError> {
    serde_json::from_value(serde_json::to_value(id).map_err(malformed)?).map_err(malformed)
}
fn raw_id(id: lsp_server::RequestId) -> std::result::Result<RequestId, TransportError> {
    serde_json::from_value(serde_json::to_value(id).map_err(malformed)?).map_err(malformed)
}
fn value(bytes: &[u8]) -> std::result::Result<serde_json::Value, TransportError> {
    serde_json::from_slice(bytes).map_err(malformed)
}
fn params(bytes: &[u8]) -> std::result::Result<serde_json::Value, TransportError> {
    if bytes.is_empty() {
        Ok(serde_json::Value::Null)
    } else {
        value(bytes)
    }
}
fn native(message: RawMessage) -> std::result::Result<lsp_server::Message, TransportError> {
    Ok(match message {
        RawMessage::Request {
            id,
            method,
            params: body,
        } => lsp_server::Request {
            id: native_id(id)?,
            method: method.into_owned(),
            params: params(&body)?,
        }
        .into(),
        RawMessage::Notification {
            method,
            params: body,
        } => lsp_server::Notification {
            method: method.into_owned(),
            params: params(&body)?,
        }
        .into(),
        RawMessage::Response { id, result } => lsp_server::Response {
            id: native_id(id)?,
            response_result: match result {
                Ok(bytes) => Ok(value(&bytes)?),
                Err(error) => Err(lsp_server::ResponseError {
                    code: error.code,
                    message: error.message,
                    data: error.data,
                }),
            },
        }
        .into(),
        RawMessage::ProtocolError { error } => {
            return Err(TransportError::Malformed(error.message));
        }
    })
}
fn raw(message: lsp_server::Message) -> std::result::Result<RawMessage, TransportError> {
    Ok(match message {
        lsp_server::Message::Request(request) => RawMessage::Request {
            id: raw_id(request.id)?,
            method: Cow::Owned(request.method),
            params: Bytes::from(serde_json::to_vec(&request.params).map_err(malformed)?),
        },
        lsp_server::Message::Notification(notification) => RawMessage::Notification {
            method: Cow::Owned(notification.method),
            params: Bytes::from(serde_json::to_vec(&notification.params).map_err(malformed)?),
        },
        lsp_server::Message::Response(response) => RawMessage::Response {
            id: raw_id(response.id)?,
            result: match response.response_result {
                Ok(value) => Ok(Bytes::from(serde_json::to_vec(&value).map_err(malformed)?)),
                Err(error) => Err(lspf::JsonRpcError {
                    code: error.code,
                    message: error.message,
                    data: error.data,
                }),
            },
        },
    })
}
pub(super) fn spawn(
    command: &mut Command,
    root: String,
    stderr: std::fs::File,
    configuration: Arc<ConfigurationExchange>,
) -> Result<(NativeTransport, ChildOwner, Arc<Mutex<Evidence>>)> {
    let mut child = ChildOwner(
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .spawn()?,
    );
    let stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| Error::Protocol("analyzer stdout is absent".into()))?;
    let mut stdin = child
        .0
        .stdin
        .take()
        .ok_or_else(|| Error::Protocol("analyzer stdin is absent".into()))?;
    let evidence = Arc::new(Mutex::new(Evidence::default()));
    let reader_evidence = evidence.clone();
    let reader_configuration = configuration.clone();
    let (incoming, messages) = mpsc::channel(256);
    std::thread::spawn(move || {
        let mut stdout = BufReader::new(stdout);
        let mut deliver = true;
        loop {
            let message = match lsp_server::Message::read(&mut stdout) {
                Ok(Some(message)) => raw(message),
                Ok(None) => {
                    let _ = incoming.blocking_send(Err(TransportError::Closed));
                    break;
                }
                Err(error) => {
                    let _ = incoming.blocking_send(Err(error.into()));
                    break;
                }
            };
            if let Ok(message) = &message
                && let Ok(mut evidence) = reader_evidence.lock()
            {
                match message {
                    RawMessage::Request { id, method, params }
                        if method == "workspace/configuration"
                            && reader_configuration.phase.load(Ordering::SeqCst) == 1 =>
                    {
                        if let Ok(params) = serde_json::from_slice::<serde_json::Value>(params)
                            && params["items"].as_array().is_some_and(|items| {
                                items.len() == 1
                                    && items[0]["scopeUri"].is_null()
                                    && items[0]["section"] == "rust-analyzer"
                            })
                            && let Ok(mut request) = reader_configuration.request.lock()
                        {
                            *request = Some(id.clone());
                        }
                    }
                    RawMessage::Response {
                        id,
                        result: Ok(bytes),
                    } if evidence.id.as_ref() == Some(id) => {
                        evidence.result = serde_json::from_slice(bytes).ok();
                    }
                    RawMessage::Notification { method, params }
                        if method == "experimental/serverStatus" =>
                    {
                        evidence.status = serde_json::from_slice(params).ok();
                    }
                    _ => {}
                }
            }
            if deliver && incoming.blocking_send(message).is_err() {
                // The library can finish its logical exit before the original backend's
                // final notifications. Retain/drain its actual stdout until original EOF.
                // Closing this pipe early would turn a clean shutdown into SIGPIPE.
                deliver = false;
            }
        }
    });
    let (outgoing, commands) = std::sync::mpsc::channel::<(
        RawMessage,
        oneshot::Sender<std::result::Result<(), TransportError>>,
    )>();
    std::thread::spawn(move || {
        while let Ok((message, reply)) = commands.recv() {
            let result = native(message)
                .and_then(|message| message.write(&mut stdin).map_err(TransportError::from));
            let failed = result.is_err();
            let _ = reply.send(result);
            if failed {
                break;
            }
        }
    });
    Ok((
        NativeTransport {
            reader: Reader { messages },
            writer: Writer {
                messages: outgoing,
                evidence: evidence.clone(),
                root,
                configuration,
            },
        },
        child,
        evidence,
    ))
}
