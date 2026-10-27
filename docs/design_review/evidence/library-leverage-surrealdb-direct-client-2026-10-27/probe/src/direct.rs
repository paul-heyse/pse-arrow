//! Minimal direct client: the generated `SurrealDbServiceClient` plus
//! `surrealdb-types` (feature `proto`) conversions. Everything below is what a
//! repository-owned facade would have to own to reproduce the SDK's observable
//! behaviour for canonical script shapes. It deliberately omits the bounded
//! lifecycle tier (admission, owned HTTP/2 driver, wire preflight, restoration).

use std::collections::BTreeMap;
use std::time::Duration;

use surrealdb_protocol::proto::rpc::v1 as rpc;
use surrealdb_protocol::proto::rpc::v1::surreal_db_service_client::SurrealDbServiceClient;
use surrealdb_protocol::proto::v1 as proto;
use surrealdb_types::prost::Message;
use surrealdb_types::{Array, Error, Object, SerializationError, SurrealValue, Value};
use tonic::transport::Channel;

pub const MESSAGE_BYTES: usize = 4 << 20;

/// What the façade reports about one unary status failure, for evidence only.
#[derive(Clone, Debug)]
pub struct StatusTrace {
    pub code: tonic::Code,
    pub has_error_bin: bool,
    pub decoded_kind: Option<String>,
    pub decoded_detail_kind: Option<String>,
    pub decoded_retryable: Option<bool>,
}

#[derive(Clone)]
pub struct Direct {
    client: SurrealDbServiceClient<Channel>,
    pub session: uuid::Uuid,
    timeout: Duration,
    pub capabilities: rpc::ServerCapabilities,
    pub last_status: std::sync::Arc<std::sync::Mutex<Option<StatusTrace>>>,
}

/// Indexed statement results with the SDK's `take` conversion rules.
#[derive(Debug, Default)]
pub struct Results {
    // Insertion-ordered by compacted index (BTreeMap keys are the indices).
    results: BTreeMap<usize, Result<Value, Error>>,
}

impl Results {
    pub fn num_statements(&self) -> usize {
        self.results.len()
    }

    pub fn take_errors(&mut self) -> BTreeMap<usize, Error> {
        let keys: Vec<usize> =
            self.results.iter().filter(|(_, r)| r.is_err()).map(|(k, _)| *k).collect();
        let mut out = BTreeMap::new();
        for key in keys {
            if let Some(Err(error)) = self.results.remove(&key) {
                out.insert(key, error);
            }
        }
        out
    }

    /// SDK rule: `take::<Value>(usize)`.
    pub fn take_value(&mut self, index: usize) -> Result<Value, Error> {
        match self.results.remove(&index) {
            Some(result) => result,
            None => Ok(Value::None),
        }
    }

    /// SDK rule: `take::<Option<T>>(usize)`.
    pub fn take_option<T: SurrealValue>(&mut self, index: usize) -> Result<Option<T>, Error> {
        let value = match self.results.remove(&index) {
            Some(Err(err)) => return Err(err),
            Some(Ok(value)) => value,
            None => return Ok(None),
        };
        let convert = |v: Value| T::from_value(v).map_err(|e| Error::internal(e.to_string()));
        match value {
            Value::Array(vec) => {
                let mut vec = vec.into_vec();
                match vec.len() {
                    0 => Ok(None),
                    1 => match vec.swap_remove(0) {
                        Value::None => Ok(None),
                        v => Ok(Some(convert(v)?)),
                    },
                    _ => Err(Error::internal(
                        "Tried to take only a single result from a query that contains multiple"
                            .to_string(),
                    )),
                }
            }
            Value::None => Ok(None),
            v => Ok(Some(convert(v)?)),
        }
    }

    /// SDK rule: `take::<Vec<T>>(usize)`.
    pub fn take_vec<T: SurrealValue>(&mut self, index: usize) -> Result<Vec<T>, Error> {
        let vec = match self.results.remove(&index) {
            Some(result) => match result? {
                Value::Array(arr) => arr.into_vec(),
                v => vec![v],
            },
            None => return Ok(vec![]),
        };
        vec.into_iter()
            .map(|v| T::from_value(v).map_err(|e| Error::internal(e.to_string())))
            .collect()
    }
}

impl Direct {
    pub async fn connect(address: std::net::SocketAddr) -> Result<Self, Error> {
        let channel = tonic::transport::Endpoint::from_shared(format!("http://{address}"))
            .map_err(|e| Error::configuration(e.to_string(), None))?
            .connect()
            .await
            .map_err(|e| Error::connection(e.to_string(), None))?;
        let mut client = SurrealDbServiceClient::new(channel)
            .max_decoding_message_size(MESSAGE_BYTES)
            .max_encoding_message_size(MESSAGE_BYTES);
        let last_status = std::sync::Arc::new(std::sync::Mutex::new(None));
        let trace = last_status.clone();
        let map = move |s: tonic::Status| status_to_error(s, &trace);
        let capabilities = client
            .get_capabilities(rpc::GetCapabilitiesRequest {
                context: None,
                client: Some(rpc::ClientInfo {
                    name: "pse-probe-direct".to_string(),
                    version: "0".to_string(),
                    platform: "rust".to_string(),
                    metadata: Vec::new(),
                }),
            })
            .await
            .map_err(&map)?
            .into_inner()
            .capabilities
            .ok_or_else(|| Error::internal("Server did not report its capabilities".into()))?;
        let session = client
            .attach_session(rpc::AttachSessionRequest { context: None })
            .await
            .map_err(&map)?
            .into_inner()
            .session
            .ok_or_else(|| Error::internal("Server did not allocate a session".into()))?
            .to_uuid()
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(Self {
            client,
            session,
            timeout: Duration::from_secs(30),
            capabilities,
            last_status,
        })
    }

    fn context(&self, transaction: Option<uuid::Uuid>) -> rpc::RequestContext {
        rpc::RequestContext {
            session: Some(proto::Uuid::from_uuid(self.session)),
            transaction: transaction.map(proto::Uuid::from_uuid),
            timeout: proto::Duration::try_from(self.timeout).ok(),
        }
    }

    fn map(&self) -> impl Fn(tonic::Status) -> Error + '_ {
        move |s| status_to_error(s, &self.last_status)
    }

    /// Root signin (the SDK's `Root { user, pass }` maps to `UserCredentials`).
    pub async fn signin_root(&self, user: &str, pass: &str) -> Result<rpc::Tokens, Error> {
        let request = rpc::SigninRequest {
            context: Some(self.context(None)),
            access_method: Some(rpc::AccessMethod {
                method: Some(rpc::access_method::Method::User(rpc::UserCredentials {
                    namespace: String::new(),
                    database: String::new(),
                    username: user.to_string(),
                    password: pass.to_string(),
                    access: String::new(),
                })),
            }),
        };
        self.client
            .clone()
            .signin(request)
            .await
            .map_err(self.map())?
            .into_inner()
            .tokens
            .ok_or_else(|| Error::internal("Server did not return any tokens".into()))
    }

    pub async fn use_ns_db(&self, ns: &str, db: &str) -> Result<(String, String), Error> {
        let nullable = |v: &str| rpc::NullableString {
            value: Some(rpc::nullable_string::Value::Some(v.to_string())),
        };
        let response = self
            .client
            .clone()
            .r#use(rpc::UseRequest {
                context: Some(self.context(None)),
                namespace: Some(nullable(ns)),
                database: Some(nullable(db)),
            })
            .await
            .map_err(self.map())?
            .into_inner();
        Ok((response.namespace, response.database))
    }

    pub async fn begin(&self) -> Result<uuid::Uuid, Error> {
        self.client
            .clone()
            .begin_transaction(rpc::BeginTransactionRequest { context: Some(self.context(None)) })
            .await
            .map_err(self.map())?
            .into_inner()
            .transaction
            .ok_or_else(|| Error::internal("Server did not return a transaction id".into()))?
            .to_uuid()
            .map_err(|e| Error::internal(e.to_string()))
    }

    pub async fn commit(&self, txn: uuid::Uuid) -> Result<(), Error> {
        self.client
            .clone()
            .commit_transaction(rpc::CommitTransactionRequest {
                context: Some(self.context(Some(txn))),
            })
            .await
            .map_err(self.map())?;
        Ok(())
    }

    /// Buffered query with the SDK's frame assembly and statement compaction.
    pub async fn query(
        &self,
        query: &str,
        variables: Vec<(&str, Value)>,
        transaction: Option<uuid::Uuid>,
    ) -> Result<Results, Error> {
        let statement_limit = query
            .len()
            .min(1 << 20)
            .min(MESSAGE_BYTES / std::mem::size_of::<Option<Statement>>());
        let variables: proto::Variables = variables
            .into_iter()
            .map(|(k, v)| {
                proto::Value::try_from(v)
                    .map(|v| (k.to_string(), v))
                    .map_err(|e| {
                        Error::serialization(e.to_string(), SerializationError::Serialization)
                    })
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .collect();
        // SDK builder rule (method::query::join_queries): each submitted query is
        // trimmed of trailing ';' and whitespace. Toggle to show the divergence.
        let submitted = if std::env::var("PROBE_TRIM").is_ok() {
            query.trim_end_matches(|c: char| c == ';' || c.is_whitespace()).to_string()
        } else {
            query.to_string()
        };
        let request = rpc::QueryRequest {
            context: Some(self.context(transaction)),
            query: submitted,
            variables: Some(variables),
            accepted_encodings: Vec::new(),
            max_batch_records: 0,
        };
        let map = self.map();
        let mut stream = self.client.clone().query(request).await.map_err(&map)?.into_inner();
        let mut statements: Vec<Option<Statement>> = Vec::new();
        let mut ended = None;
        let mut began = false;
        while let Some(response) = stream.message().await.map_err(&map)? {
            match response.frame {
                Some(rpc::query_response::Frame::Begin(begin)) => {
                    if began {
                        return Err(Error::connection("Query repeated its Begin frame".into(), None));
                    }
                    began = true;
                    let len = begin.statement_count as usize;
                    if len > statement_limit {
                        return Err(Error::connection(
                            format!("statement count {len} exceeds allowance {statement_limit}"),
                            None,
                        ));
                    }
                    statements.resize_with(len, || None);
                }
                Some(rpc::query_response::Frame::Batch(batch)) => {
                    let index = batch.query_index as usize;
                    if !began || index >= statements.len() {
                        return Err(Error::connection(
                            "Query statement index exceeds its Begin count".into(),
                            None,
                        ));
                    }
                    statements[index].get_or_insert_with(Statement::default).push(batch);
                }
                Some(rpc::query_response::Frame::End(end)) => {
                    ended = Some(end);
                    break;
                }
                Some(rpc::query_response::Frame::Error(error)) => {
                    let error = proto_error(error);
                    if stream.message().await.map_err(&map)?.is_some() {
                        return Err(Error::connection(
                            "Query carried frames after terminal failure".into(),
                            None,
                        ));
                    }
                    return Err(error);
                }
                None => {
                    return Err(Error::internal("Query stream carried an unrecognised frame".into()));
                }
            }
        }
        let Some(end) = ended else {
            return Err(Error::connection(
                "The query ended before it was complete".into(),
                surrealdb_types::ConnectionError::ConnectionFailed,
            ));
        };
        if statements.iter().flatten().any(|s| !s.complete) {
            return Err(Error::connection(
                "Query ended before every statement completed".into(),
                None,
            ));
        }
        if stream.message().await.map_err(&map)?.is_some() {
            return Err(Error::connection("Query carried frames after completion".into(), None));
        }
        // Compaction: indices that never emitted are dropped, then re-enumerated,
        // exactly like `finish_statements` + `IndexedResults` assembly.
        let finished: Vec<(i32, Result<Value, Error>)> =
            statements.into_iter().flatten().map(Statement::finish).collect();
        if finished.len() != end.result_count as usize {
            return Err(Error::connection(
                format!(
                    "The query reported {} statement results but {} arrived",
                    end.result_count,
                    finished.len()
                ),
                surrealdb_types::ConnectionError::ConnectionFailed,
            ));
        }
        let mut results = Results::default();
        for (index, (kind, result)) in finished.into_iter().enumerate() {
            // QueryType::Kill results are dropped by the SDK; Live is out of scope here.
            if kind == rpc::QueryStatementKind::Kill as i32 {
                continue;
            }
            results.results.insert(index, result);
        }
        Ok(results)
    }
}

#[derive(Default)]
struct Statement {
    complete: bool,
    batches: u64,
    values: Vec<Value>,
    single: bool,
    statement_kind: i32,
    error: Option<Error>,
}

impl Statement {
    fn push(&mut self, batch: rpc::QueryBatchFrame) {
        if self.complete || batch.batch_index != self.batches {
            self.error = Some(Error::connection(
                "Query batch sequence is incomplete or repeated".into(),
                None,
            ));
            return;
        }
        self.batches += 1;
        self.complete = batch.error.is_some()
            || matches!(
                rpc::QueryResponseKind::try_from(batch.kind),
                Ok(rpc::QueryResponseKind::Single | rpc::QueryResponseKind::BatchedFinal)
            );
        self.statement_kind = batch.statement_kind;
        if batch.kind == rpc::QueryResponseKind::Single as i32 {
            self.single = true;
        }
        if let Some(error) = batch.error {
            self.error = Some(proto_error(error));
            return;
        }
        match batch.payload {
            Some(rpc::query_batch_frame::Payload::Values(values)) => {
                for value in values.values {
                    match Value::try_from(value) {
                        Ok(value) => self.values.push(value),
                        Err(e) => {
                            self.error = Some(Error::serialization(
                                e.to_string(),
                                SerializationError::Deserialization,
                            ));
                            return;
                        }
                    }
                }
            }
            Some(rpc::query_batch_frame::Payload::Arrow(_)) => {
                self.error = Some(Error::internal(
                    "Server sent a columnar batch, which was not requested".into(),
                ));
            }
            None => {}
        }
    }

    fn finish(self) -> (i32, Result<Value, Error>) {
        let result = match self.error {
            Some(error) => Err(error),
            None if self.single => Ok(self.values.into_iter().next().unwrap_or(Value::None)),
            None => Ok(Value::Array(Array::from(self.values))),
        };
        (self.statement_kind, result)
    }
}

/// Unary failure: typed trailer first, status code mapping second.
fn status_to_error(
    status: tonic::Status,
    trace: &std::sync::Mutex<Option<StatusTrace>>,
) -> Error {
    let mut record = StatusTrace {
        code: status.code(),
        has_error_bin: false,
        decoded_kind: None,
        decoded_detail_kind: None,
        decoded_retryable: None,
    };
    if let Some(encoded) = status.metadata().get_bin("surreal-error-bin") {
        record.has_error_bin = true;
        const LIMIT: usize = 8 << 10;
        let decoded = (|| -> Result<Error, Error> {
            let bytes = encoded
                .to_bytes()
                .map_err(|e| Error::serialization(e.to_string(), SerializationError::Deserialization))?;
            if bytes.len() > LIMIT {
                return Err(Error::serialization(
                    "native error metadata exceeds header envelope".into(),
                    SerializationError::Deserialization,
                ));
            }
            let error = proto::SurrealError::decode(bytes)
                .map_err(|e| Error::serialization(e.to_string(), SerializationError::Deserialization))?;
            record.decoded_kind = Some(format!("{:?}", error.kind_or_internal()));
            record.decoded_detail_kind = error.detail_kind().map(str::to_string);
            record.decoded_retryable = Some(error.is_retryable());
            Ok(proto_error(error))
        })();
        *trace.lock().unwrap() = Some(record);
        return decoded.unwrap_or_else(|e| e);
    }
    *trace.lock().unwrap() = Some(record);
    use tonic::Code;
    let message = status.message().to_string();
    match status.code() {
        Code::InvalidArgument | Code::OutOfRange => Error::validation(message, None),
        Code::Unimplemented => Error::configuration(message, None),
        Code::DeadlineExceeded | Code::Aborted => Error::query(message, None),
        Code::FailedPrecondition => {
            if status.metadata().contains_key("surreal-rejected-before-execution") {
                Error::validation(message, None)
            } else {
                Error::query(message, None)
            }
        }
        Code::PermissionDenied | Code::Unauthenticated => Error::not_allowed(message, None),
        Code::NotFound => Error::not_found(message, None),
        Code::AlreadyExists => Error::already_exists(message, None),
        Code::Unavailable | Code::Cancelled | Code::Unknown => Error::connection(message, None),
        _ => Error::internal(message),
    }
}

fn kind_str(kind: proto::ErrorKind) -> &'static str {
    use proto::ErrorKind;
    match kind {
        ErrorKind::Validation => "Validation",
        ErrorKind::Configuration => "Configuration",
        ErrorKind::Query => "Query",
        ErrorKind::Serialization => "Serialization",
        ErrorKind::NotAllowed => "NotAllowed",
        ErrorKind::NotFound => "NotFound",
        ErrorKind::AlreadyExists => "AlreadyExists",
        ErrorKind::Connection => "Connection",
        ErrorKind::Thrown => "Thrown",
        ErrorKind::Context => "Context",
        ErrorKind::Internal | ErrorKind::Unspecified => "Internal",
    }
}

fn proto_details(details: proto::ErrorDetails) -> Option<Value> {
    if details.kind.is_empty() {
        return None;
    }
    let mut object = Object::new();
    object.insert("kind".to_string(), Value::String(details.kind));
    if let Some(content) = details.content.and_then(|c| Value::try_from(c).ok()) {
        object.insert("details".to_string(), content);
    }
    Some(Value::Object(object))
}

pub fn proto_error(error: proto::SurrealError) -> Error {
    let kind = error.kind_or_internal();
    let cause = error.cause.map(|cause| proto_error(*cause));
    let details = error.details.and_then(proto_details);
    let mapped = Error::from_wire(error.message, Some(kind_str(kind)), details);
    match cause {
        Some(cause) => mapped.with_cause(cause),
        None => mapped,
    }
}
