// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Concrete remote canonical substrate. Scientific work runs outside guarded transactions.

use crate::{canonical_codec::CodecError, generated::surreal as wire};
pub use pse_model::generated::runtime::{
    canonical_memberships::Row as Membership, canonical_revisions::Row as Revision,
    canonical_versions::Row as ObjectVersion,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use surrealdb::{
    Surreal,
    engine::remote::grpc::{Client, Grpc},
    opt::{Config, GrpcConfig, auth::Root},
    types::{Object, Value},
};

/// Initial negotiated protocol limit; batches must leave space for their envelope.
pub const MESSAGE_BYTES: usize = 4 * 1024 * 1024;
/// Bounded submitted payload, leaving conservative room for query/record metadata.
pub const PAYLOAD_BYTES: usize = 3 * 1024 * 1024;
/// Client operation deadline. Mutations settle a lost acknowledgment by immutable identity.
pub const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const ACTIVATION_QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
const ACTIVATION_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(90);

/// Canonical protocol and guarded-operation failures.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(runtime::infrastructure))]
pub enum CanonicalError {
    #[error("canonical store configuration: {0}")]
    /// Invalid deployment or operation configuration.
    Configuration(String),
    #[error("canonical store driver: {0}")]
    /// Protocol/database failure retaining its native typed cause.
    Driver(#[from] surrealdb::Error),
    #[error(transparent)]
    /// Declared wire domain or shape refusal.
    Codec(#[from] CodecError),
    #[error("canonical interpretation mismatch: expected {expected}, observed {observed}")]
    /// Installed scientific interpretation differs from this reader.
    Interpretation {
        /// Required immutable interpretation.
        expected: &'static str,
        /// Recorded interpretation or absence.
        observed: String,
    },
    #[error("canonical immutable operation identity was reused for different inputs")]
    /// An idempotency identity was reused for different supplied bytes.
    OperationReused,
    #[error("canonical write payload exceeds the bounded protocol batch")]
    /// A bounded payload or selector limit was exceeded.
    PayloadLimit,
    #[error("canonical store is quiesced")]
    /// The supervisor has closed write admission.
    Quiesced,
    #[error("canonical store request deadline expired")]
    /// No success is inferred from a timed-out request.
    Timeout,
    #[error("canonical query returned no statement completion")]
    /// A cancelled or incomplete query cannot establish data absence or write success.
    IncompleteResponse,
}
impl pse_diagnostics::TypedDiagnostic for CanonicalError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        Some(match self {
            Self::Codec(_) => pse_diagnostics::DiagnosticCode::ValidationInvariant,
            _ => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
        })
    }
}

/// Authenticated deployment parameters. Credentials never appear in Debug or status output.
pub struct CanonicalOptions {
    /// Authenticated loopback gRPC endpoint.
    pub endpoint: String,
    /// Application namespace.
    pub namespace: String,
    /// Application or isolated fixture database.
    pub database: String,
    /// Authentication role, omitted from Debug.
    pub username: String,
    /// Authentication secret, omitted from Debug.
    pub password: String,
    /// Finite managed native-worker allocation read from the owning supervisor profile.
    pub native: NativeAllocation,
    state_path: PathBuf,
}
/// The application profile's separately capped native-worker slots.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
pub struct NativeAllocation {
    /// Maximum managed worker processes sharing this store.
    pub native_workers: usize,
    /// Hard process memory cap for each worker.
    pub native_worker_memory_bytes: usize,
}
impl std::fmt::Debug for CanonicalOptions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CanonicalOptions")
            .field("endpoint", &self.endpoint)
            .field("namespace", &self.namespace)
            .field("database", &self.database)
            .finish_non_exhaustive()
    }
}
impl CanonicalOptions {
    /// Read the supervisor's private state, including its admission gate.
    pub fn from_state(state: &Path) -> Result<Self, CanonicalError> {
        #[derive(serde::Deserialize)]
        struct Deployment {
            endpoint: String,
            namespace: String,
            database: String,
            schema_interpretation: String,
            max_message_bytes: usize,
            credentials_file: String,
            resources: NativeAllocation,
        }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Credentials {
            username: String,
            password: String,
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let directory = std::fs::symlink_metadata(state)
                .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
            if !directory.is_dir()
                || directory.file_type().is_symlink()
                || directory.mode() & 0o077 != 0
            {
                return Err(CanonicalError::Configuration(
                    "canonical state must be a private real directory".into(),
                ));
            }
            #[cfg(target_os = "linux")]
            if directory.uid()
                != std::fs::metadata("/proc/self")
                    .map_err(|e| CanonicalError::Configuration(e.to_string()))?
                    .uid()
            {
                return Err(CanonicalError::Configuration(
                    "canonical state must belong to the current application user".into(),
                ));
            }
        }
        let read = |path: &Path| -> Result<Vec<u8>, CanonicalError> {
            let metadata = std::fs::symlink_metadata(path)
                .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() > 64 * 1024
            {
                return Err(CanonicalError::Configuration(
                    "canonical configuration must be a bounded real file".into(),
                ));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if metadata.mode() & 0o077 != 0
                    || metadata.uid()
                        != std::fs::metadata(state)
                            .map_err(|e| CanonicalError::Configuration(e.to_string()))?
                            .uid()
                {
                    return Err(CanonicalError::Configuration("canonical configuration and credentials must be private and share the state owner".into()));
                }
            }
            std::fs::read(path).map_err(|error| CanonicalError::Configuration(error.to_string()))
        };
        let deployment: Deployment = serde_json::from_slice(&read(&state.join("config.json"))?)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        if deployment.schema_interpretation != wire::INTERPRETATION {
            return Err(CanonicalError::Interpretation {
                expected: wire::INTERPRETATION,
                observed: deployment.schema_interpretation,
            });
        }
        if deployment.max_message_bytes != MESSAGE_BYTES {
            return Err(CanonicalError::Configuration(
                "unsupported protocol bound".into(),
            ));
        }
        if !(1..=32).contains(&deployment.resources.native_workers)
            || deployment.resources.native_worker_memory_bytes == 0
        {
            return Err(CanonicalError::Configuration(
                "finite native-worker allocation required".into(),
            ));
        }
        let credentials: Credentials =
            serde_json::from_slice(&read(Path::new(&deployment.credentials_file))?)
                .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        Ok(Self {
            endpoint: deployment.endpoint,
            namespace: deployment.namespace,
            database: deployment.database,
            username: credentials.username,
            password: credentials.password,
            native: deployment.resources,
            state_path: state.to_owned(),
        })
    }
}

/// Immutable authored mutation. A removal has no new version or references.
#[derive(Clone, Debug)]
pub struct ObjectEdit {
    /// Stable authored logical object identity.
    pub logical: String,
    /// Lexical namespace of this object.
    pub scope: String,
    /// Authored name within the lexical namespace.
    pub name: String,
    /// New immutable version; absent for removal.
    pub version: Option<ObjectVersion>,
    /// Structural reference scope/name pairs.
    pub references: Vec<(String, String)>,
}

/// A store-issued immutable preparation/read protection; its expiry is checked server-side.
#[derive(Clone, Debug)]
pub struct ProtectedSelection {
    key: String,
    revision: Revision,
}
impl ProtectedSelection {
    pub(crate) fn key(&self) -> &str {
        &self.key
    }
    /// Exact canonical revision retained by this protection.
    pub fn revision(&self) -> &Revision {
        &self.revision
    }
}

#[derive(Clone)]
/// Thin remote client; clones share its bounded transport.
pub struct CanonicalStore {
    pub(crate) db: Arc<Surreal<Client>>,
    #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
    pub(crate) result_read_drain: Arc<crate::canonical_results::ResultReadDrain>,
    activation_db: Arc<Surreal<Client>>,
    state_path: PathBuf,
    database: String,
}
impl std::fmt::Debug for CanonicalStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CanonicalStore")
            .field("state", &self.state_path)
            .finish_non_exhaustive()
    }
}
impl CanonicalStore {
    /// Exact configured canonical database selected by this deployment handle.
    pub fn database(&self) -> &str {
        &self.database
    }
    /// Refuse new application writes and native claims while the supervisor is draining.
    pub fn check_write_admission(&self) -> Result<(), CanonicalError> {
        self.ensure_writes()
    }
    pub(crate) fn ensure_writes(&self) -> Result<(), CanonicalError> {
        #[derive(serde::Deserialize)]
        struct Admission {
            accepting_writes: bool,
        }
        let config = std::fs::read(self.state_path.join("config.json"))
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        let admission: Admission = serde_json::from_slice(&config)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        if !admission.accepting_writes {
            return Err(CanonicalError::Quiesced);
        }
        Ok(())
    }
    /// Connect to the supported authenticated gRPC deployment.
    pub async fn connect(options: &CanonicalOptions) -> Result<Self, CanonicalError> {
        let endpoint = url::Url::parse(&options.endpoint)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        if endpoint.scheme() != "grpc"
            || !matches!(endpoint.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
        {
            return Err(CanonicalError::Configuration(
                "initial supported profile requires loopback grpc".into(),
            ));
        }
        let address = endpoint
            .socket_addrs(|| None)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            .into_iter()
            .next()
            .ok_or_else(|| CanonicalError::Configuration("endpoint has no port".into()))?;
        let db = connect_client(address, options, std::time::Duration::from_secs(20)).await?;
        let activation_db = connect_client(address, options, ACTIVATION_QUERY_TIMEOUT).await?;
        Ok(Self {
            db,
            #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
            result_read_drain: Arc::new(crate::canonical_results::ResultReadDrain::default()),
            activation_db,
            state_path: options.state_path.clone(),
            database: options.database.clone(),
        })
    }
    /// Explicit schema creation; ordinary opening never mutates an unknown schema.
    pub async fn create(&self) -> Result<(), CanonicalError> {
        self.ensure_writes()?;
        // Refuse an unmarked partial database rather than declaring imported records ready.
        let mut inventory = bounded_query(self.db.query("INFO FOR DB;")).await?;
        let info: Value = inventory.take(0)?;
        let Value::Object(info) = info else {
            return Err(CanonicalError::Configuration(
                "database inventory is not an object".into(),
            ));
        };
        if let Some(Value::Object(tables)) = info.get("tables") {
            if tables.contains_key("canonical_interpretations") {
                return self.open().await;
            }
            if tables.keys().any(|name| name.starts_with("canonical_")) {
                return Err(CanonicalError::Configuration(
                    "unmarked canonical tables require explicit rebuild".into(),
                ));
            }
        }
        let response = request(self.db.query(format!("BEGIN;\n{}\nCREATE canonical_interpretations:current SET key = 'current', interpretation = $interpretation, schema_digest = $schema_digest;\nCOMMIT;", wire::SCHEMA)).bind(("interpretation", wire::INTERPRETATION)).bind(("schema_digest", wire::SCHEMA_DIGEST))).await?;
        checked(response)?;
        Ok(())
    }
    /// Read and verify the installed interpretation without DDL.
    pub async fn open(&self) -> Result<(), CanonicalError> {
        let marker: Option<Object> =
            request(self.db.select(("canonical_interpretations", "current"))).await?;
        let Some(marker) = marker else {
            return Err(CanonicalError::Interpretation {
                expected: wire::INTERPRETATION,
                observed: "absent".into(),
            });
        };
        let marker = wire::decode_canonical_interpretations(marker)?;
        if marker.interpretation != wire::INTERPRETATION {
            return Err(CanonicalError::Interpretation {
                expected: wire::INTERPRETATION,
                observed: marker.interpretation,
            });
        }
        if marker.schema_digest != wire::SCHEMA_DIGEST {
            return Err(CanonicalError::Interpretation {
                expected: wire::SCHEMA_DIGEST,
                observed: marker.schema_digest,
            });
        }
        Ok(())
    }
    /// Exact canonical revision retained by this protection.
    pub async fn revision(&self, id: &str) -> Result<Option<Revision>, CanonicalError> {
        let row: Option<Object> = request(self.db.select(("canonical_revisions", id))).await?;
        row.map(wire::decode_canonical_revisions)
            .transpose()
            .map_err(Into::into)
    }
    /// Atomic head-CAS edit. The operation ID settles an acknowledgment lost after commit.
    pub async fn edit(
        &self,
        problem: &str,
        expected: Option<&str>,
        operation: &str,
        edits: &[ObjectEdit],
    ) -> Result<Revision, CanonicalError> {
        self.ensure_writes()?;
        let stage = match self
            .stage_edits(problem, expected, operation, edits)
            .await?
        {
            crate::canonical_staging::StageOutcome::Acknowledged(revision) => return Ok(revision),
            crate::canonical_staging::StageOutcome::Closed(stage) => stage,
        };
        for attempt in 0..8 {
            self.ensure_writes()?;
            let query = self
                .activation_db
                .query(crate::canonical_staging::bounded_edit_sql(EDIT))
                .bind(("problem", stage.problem.clone()))
                .bind(("expected", stage.expected.clone()))
                .bind(("operation", stage.operation.clone()))
                .bind(("interpretation", wire::INTERPRETATION))
                .bind((
                    "request",
                    surrealdb::types::Bytes::from(stage.request.clone()),
                ))
                .bind(("request_digest", stage.request_digest.clone()))
                .bind((
                    "generation",
                    crate::canonical_codec::encode_uint(stage.generation)?,
                ));
            let result = tokio::time::timeout(ACTIVATION_REQUEST_TIMEOUT, query.into_future())
                .await
                .map_err(|_| CanonicalError::Timeout)
                .and_then(|response| response.map_err(CanonicalError::from))
                .and_then(complete_response);
            if let Err(error) = result {
                // A lost acknowledgment or concurrent identical activation is settled
                // exclusively by the immutable operation receipt, never by the head.
                if let Ok(Some(revision)) = self.revision(operation).await {
                    return if revision.request.as_slice() == stage.request.as_slice() {
                        Ok(revision)
                    } else {
                        Err(CanonicalError::OperationReused)
                    };
                }
                if matches!(&error, CanonicalError::Driver(error) if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)))
                    && attempt < 7
                {
                    tokio::task::yield_now().await;
                    continue;
                }
                return Err(error);
            }
            let revision = self.revision(operation).await?;
            return revision
                .ok_or_else(|| CanonicalError::Configuration("committed revision missing".into()));
        }
        Err(CanonicalError::Configuration(
            "staged activation retries exhausted".into(),
        ))
    }
    /// Explicit historical inventory, indexed by problem and revision interval.
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) async fn inventory(
        &self,
        revision: &Revision,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let mut response = bounded_query(self.db.query("SELECT * FROM canonical_memberships WHERE problem = $problem AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) ORDER BY scope, name;")
            .bind(("problem", revision.problem.clone())).bind(("sequence", crate::canonical_codec::encode_uint(revision.sequence)?))).await?;
        let rows: Vec<Object> = response.take(0)?;
        rows.into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
    /// Read an immutable version by exact identity.
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) async fn object(
        &self,
        version: &str,
    ) -> Result<Option<ObjectVersion>, CanonicalError> {
        self.assemble_object(version).await
    }
    /// Protect before off-transaction scientific work. Server time controls expiration.
    pub async fn protect(
        &self,
        revision: Revision,
        lifetime: std::time::Duration,
    ) -> Result<ProtectedSelection, CanonicalError> {
        // Read protections are lease bookkeeping, including gated restore
        // verification. Scientific mutation and reclamation remain quiesced.
        let micros = i64::try_from(lifetime.as_micros())
            .map_err(|_| CanonicalError::Configuration("protection lifetime overflow".into()))?;
        if micros <= 0 {
            return Err(CanonicalError::Configuration(
                "positive protection lifetime required".into(),
            ));
        }
        let key = uuid::Uuid::new_v4().to_string();
        let mut response = protected_query(|| Ok(self.db.query(r#"BEGIN;
            SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE;
            LET $revision = SELECT * FROM ONLY type::record('canonical_revisions', $revision);
            IF $revision = NONE OR $revision.problem != $problem { THROW 'revision unavailable'; };
            LET $reclaimed = SELECT key FROM canonical_reclaimed_ranges WHERE problem = $problem AND from_sequence <= $revision.sequence AND to_sequence > $revision.sequence LIMIT 1;
            IF array::len($reclaimed) != 0 { THROW 'source selection reclaimed'; };
            IF (SELECT * FROM ONLY type::record('canonical_protections', $key)) = NONE { CREATE type::record('canonical_protections', $key) SET key = $key, problem = $problem, revision = $revision.key, sequence = $revision.sequence, expires_at = time::micros() + $lifetime, released = false; };
            UPSERT type::record('canonical_guards', 'retention:' + $problem) SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
            SELECT * FROM ONLY type::record('canonical_revisions', $revision.key);
            COMMIT;"#).bind(("problem", revision.problem.clone())).bind(("revision", revision.key.clone())).bind(("key", key.clone())).bind(("lifetime", micros)))).await?;
        let actual: Option<Object> = response.take(response.num_statements().saturating_sub(2))?;
        let revision = wire::decode_canonical_revisions(actual.ok_or_else(|| {
            CanonicalError::Configuration("protected revision unavailable".into())
        })?)?;
        Ok(ProtectedSelection { key, revision })
    }
    /// Resolve a bounded namespace/name premise in the exact protected revision.
    pub async fn select_names(
        &self,
        selection: &ProtectedSelection,
        scope: &str,
        names: &[String],
    ) -> Result<Vec<Membership>, CanonicalError> {
        if names.len() > 256 {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut response = protected_query(|| Ok(self.db.query(format!("{}\nSELECT * FROM canonical_memberships WHERE problem = $problem AND scope = $scope AND name IN $names AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence);\nCOMMIT;", PROTECTED_BEGIN))
            .bind(("problem", selection.revision.problem.clone())).bind(("revision", selection.revision.key.clone())).bind(("protection", selection.key.clone())).bind(("scope", scope.to_owned())).bind(("names", names.to_vec())).bind(("sequence", crate::canonical_codec::encode_uint(selection.revision.sequence)?)))).await?;
        let rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
        rows.into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
    /// Product and revision roots become durable before the caller explicitly ends
    /// the selection protection. Multiple body products may share the same selection.
    pub(crate) async fn admit_product(
        &self,
        selection: &ProtectedSelection,
        product: &pse_model::generated::runtime::canonical_products::Row,
        stage: &crate::canonical_staging::ClosedStage,
    ) -> Result<String, CanonicalError> {
        self.ensure_writes()?;
        if product.problem != selection.revision.problem
            || product.revision != selection.revision.key
            || product.interpretation != wire::INTERPRETATION
        {
            return Err(CanonicalError::Configuration(
                "product selection/interpretation mismatch".into(),
            ));
        }
        if product
            .payload
            .len()
            .saturating_add(product.dependencies.len())
            .saturating_add(product.request.len())
            > PAYLOAD_BYTES
        {
            return Err(CanonicalError::PayloadLimit);
        }
        for attempt in 0..8 {
            self.ensure_writes()?;
            let result = request(
                self.db
                    .query(format!("{PROTECTED_BEGIN}\n{ADMIT_PRODUCT}\nCOMMIT;"))
                    .bind(("problem", selection.revision.problem.clone()))
                    .bind(("revision", selection.revision.key.clone()))
                    .bind(("protection", selection.key.clone()))
                    .bind((
                        "sequence",
                        crate::canonical_codec::encode_uint(selection.revision.sequence)?,
                    ))
                    .bind(("product", wire::encode_canonical_products(product)?))
                    .bind(("operation", stage.operation.clone()))
                    .bind((
                        "generation",
                        crate::canonical_codec::encode_uint(stage.generation)?,
                    ))
                    .bind(("request_digest", stage.request_digest.clone()))
                    .bind((
                        "blob",
                        crate::canonical_staging::ProductBlob::decode(product.payload.as_slice())?
                            .version,
                    ))
                    .bind((
                        "blob_digest",
                        crate::canonical_staging::ProductBlob::decode(product.payload.as_slice())?
                            .digest,
                    ))
                    .bind((
                        "blob_bytes",
                        crate::canonical_codec::encode_uint(
                            crate::canonical_staging::ProductBlob::decode(
                                product.payload.as_slice(),
                            )?
                            .bytes as u64,
                        )?,
                    )),
            )
            .await
            .and_then(|response| checked(response).map_err(Into::into));
            match result {
                Ok(mut response) => {
                    return Ok(crate::canonical_codec::decode_string(
                        response.take::<Value>(response.num_statements().saturating_sub(2))?,
                    )?);
                }
                Err(error) => {
                    // This immutable acknowledgment remains meaningful even if the
                    // preparation pin expired after the successful commit.
                    if let Ok(Some(key)) = self.product_acknowledged(product).await {
                        return Ok(key);
                    }
                    if matches!(&error, CanonicalError::Driver(error) if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)))
                        && attempt < 7
                    {
                        tokio::task::yield_now().await;
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        Err(CanonicalError::Configuration(
            "product admission retries exhausted".into(),
        ))
    }
    pub(crate) async fn product_acknowledged(
        &self,
        product: &pse_model::generated::runtime::canonical_products::Row,
    ) -> Result<Option<String>, CanonicalError> {
        // Admission can reuse a previously rooted exact product rather than create
        // the proposed publication key. Settle both branches after a lost response,
        // without requiring a still-live preparation pin or admitting anything new.
        let mut response = protected_query(|| Ok(self.db.query("BEGIN; SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE; LET $saved = SELECT key FROM canonical_products WHERE problem = $problem AND request = $product.request AND payload = $product.payload AND dependencies = $product.dependencies AND producer = $product.producer AND interpretation = $product.interpretation AND type::record('canonical_roots', key).problem = problem AND type::record('canonical_roots', key).revision = revision AND type::record('canonical_roots', key).owner_kind = 'product' AND type::record('canonical_roots', key).owner = key ORDER BY key LIMIT 1; RETURN IF array::len($saved) = 0 { NONE } ELSE { $saved[0].key }; COMMIT;")
            .bind(("problem", product.problem.clone())).bind(("product", wire::encode_canonical_products(product)?)))).await?;
        let value = response.take::<Value>(response.num_statements().saturating_sub(2))?;
        if matches!(value, Value::None) {
            return Ok(None);
        }
        Ok(Some(crate::canonical_codec::decode_string(value)?))
    }
    /// End a selection protection under its retention conflict guard.
    pub async fn release(&self, selection: &ProtectedSelection) -> Result<(), CanonicalError> {
        // Quiescing scientific writes must still let in-flight readers drain.
        protected_query(|| Ok(self.db.query("BEGIN; SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE; UPDATE type::record('canonical_protections', $protection) SET released = true; UPSERT type::record('canonical_guards', 'retention:' + $problem) SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec; COMMIT;")
            .bind(("problem", selection.revision.problem.clone())).bind(("protection", selection.key.clone())))).await?;
        Ok(())
    }
    /// Remove only this handle's explicitly isolated endpoint fixture database.
    #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
    pub async fn remove_isolated_fixture(&self) -> Result<(), CanonicalError> {
        if !self.database.starts_with("canonical_test_") {
            return Err(CanonicalError::Configuration(
                "fixture removal requires an isolated test database".into(),
            ));
        }
        self.result_read_drain.drain().await?;
        bounded_query(
            self.db
                .query("REMOVE DATABASE $database;")
                .bind(("database", self.database.clone())),
        )
        .await?;
        Ok(())
    }
}

/// Bound one native client operation, preserving its typed failure.
pub(crate) async fn request<F, T>(future: F) -> Result<T, CanonicalError>
where
    F: IntoFuture<Output = Result<T, surrealdb::Error>>,
{
    tokio::time::timeout(REQUEST_TIMEOUT, future.into_future())
        .await
        .map_err(|_| CanonicalError::Timeout)?
        .map_err(Into::into)
}

// Transaction failures also mark preceding statements NotExecuted. Preserve the substantive failure.
/// A single checked RPC with a transport deadline; callers retry complete decisions.
pub(crate) async fn bounded_query<F>(future: F) -> Result<surrealdb::IndexedResults, CanonicalError>
where
    F: IntoFuture<Output = Result<surrealdb::IndexedResults, surrealdb::Error>>,
{
    complete_response(request(future).await?)
}

fn complete_response(
    response: surrealdb::IndexedResults,
) -> Result<surrealdb::IndexedResults, CanonicalError> {
    let response = checked(response)?;
    if response.num_statements() == 0 {
        return Err(CanonicalError::IncompleteResponse);
    }
    Ok(response)
}

async fn connect_client(
    address: std::net::SocketAddr,
    options: &CanonicalOptions,
    query_timeout: std::time::Duration,
) -> Result<Arc<Surreal<Client>>, CanonicalError> {
    let config = Config::new()
        .query_timeout(query_timeout)
        .grpc(GrpcConfig::new().max_message_size(MESSAGE_BYTES))?;
    let db = request(Surreal::new::<Grpc>((address, config))).await?;
    request(db.signin(Root {
        username: options.username.clone(),
        password: options.password.clone(),
    }))
    .await?;
    request(db.use_ns(&options.namespace).use_db(&options.database)).await?;
    Ok(Arc::new(db))
}

/// Rebuild a protected immutable read or idempotent lease transaction after a
/// conflict. Every attempt rechecks the live pin and reclamation guard; no
/// scientific decision or partial response is retained across attempts.
pub(crate) async fn protected_query<F, Q>(
    mut build: F,
) -> Result<surrealdb::IndexedResults, CanonicalError>
where
    F: FnMut() -> Result<Q, CanonicalError>,
    Q: IntoFuture<Output = Result<surrealdb::IndexedResults, surrealdb::Error>>,
{
    for attempt in 0..8 {
        match bounded_query(build()?).await {
            Err(CanonicalError::Driver(error))
                if matches!(
                    error.query_details(),
                    Some(surrealdb::types::QueryError::TransactionConflict)
                ) && attempt < 7 =>
            {
                tokio::time::sleep(std::time::Duration::from_millis(1 << attempt)).await;
            }
            result => return result,
        }
    }
    Err(CanonicalError::Configuration(
        "guarded operation retries exhausted".into(),
    ))
}

pub(crate) fn checked(
    mut response: surrealdb::IndexedResults,
) -> Result<surrealdb::IndexedResults, surrealdb::Error> {
    let errors = response.take_errors().into_values().collect::<Vec<_>>();
    if !errors.is_empty() {
        let substantive = errors.iter().position(|error| {
            !matches!(
                error.query_details(),
                Some(
                    surrealdb::types::QueryError::NotExecuted
                        | surrealdb::types::QueryError::Cancelled
                )
            )
        });
        if let Some(error) = errors.into_iter().nth(substantive.unwrap_or(0)) {
            return Err(error);
        }
    }
    Ok(response)
}

// The gRPC indexed route includes BEGIN and COMMIT result slots. Consumer SELECT
// is immediately before COMMIT; do not decode either control's NONE as an object.
pub(crate) const PROTECTED_BEGIN: &str = r#"BEGIN;
SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE;
LET $pin = SELECT * FROM ONLY type::record('canonical_protections', $protection);
IF $pin = NONE OR $pin.problem != $problem OR $pin.revision != $revision OR $pin.sequence != $sequence OR $pin.released OR $pin.expires_at <= time::micros() { THROW 'immutable selection protection expired'; };"#;

const ADMIT_PRODUCT: &str = r#"
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE;
LET $version_guard = type::record('canonical_guards', 'version:' + $blob);
SELECT * FROM $version_guard FOR UPDATE;
LET $stage = SELECT * FROM ONLY type::record('canonical_stages', $operation);
LET $child = SELECT * FROM ONLY type::record('canonical_staged_edits', $operation + ':0');
LET $manifest = SELECT * FROM ONLY type::record('canonical_version_manifests', $blob);
IF $operation != $product.key OR $stage = NONE OR $stage.problem != $problem OR $stage.request_digest != $request_digest OR $stage.generation != $generation OR $stage.closed = false OR $stage.activated OR $stage.abandoned OR $stage.expires_at <= time::micros() OR $stage.edit_count != 1dec OR $stage.expected_head != NONE { THROW 'closed product staging manifest fenced'; };
IF $child = NONE OR $child.stage != $operation OR $child.ordinal != 0dec OR $child.version != $blob OR $child.logical != 'pse.product-blob.v1' OR $child.scope != 'pse.product-blob.v1' OR $child.name != 'pse.product-blob.v1' OR $manifest = NONE OR $manifest.closed = false OR $manifest.logical != 'pse.product-blob.v1' OR $manifest.kind != 'pse.product-blob.v1' OR $manifest.interpretation != $product.interpretation OR $manifest.reference_count != 0dec OR $manifest.payload_digest != $blob_digest OR $manifest.payload_len != $blob_bytes { THROW 'product blob association invalid'; };
// Repeated preparations retain one already rooted exact description. Publication
// identity is distinct from its request, and a released publication is never revived.
LET $equivalent = SELECT key FROM canonical_products WHERE problem = $problem AND producer = $product.producer AND request = $product.request AND payload = $product.payload AND dependencies = $product.dependencies AND interpretation = $product.interpretation AND type::record('canonical_roots', key).owner_kind = 'product' AND type::record('canonical_roots', key).revision = revision AND type::record('canonical_roots', key).problem = problem AND type::record('canonical_roots', key).owner = key ORDER BY key LIMIT 1;
IF array::len($equivalent) = 0 {
    LET $existing = SELECT * FROM ONLY type::record('canonical_products', $product.key);
    IF $existing = NONE {
        CREATE type::record('canonical_products', $product.key) CONTENT $product;
        CREATE type::record('canonical_roots', $product.key) SET key = $product.key, problem = $problem, revision = $product.revision, sequence = $sequence, owner_kind = 'product', owner = $product.key;
    } ELSE {
        IF $existing.problem != $product.problem OR $existing.request != $product.request OR $existing.payload != $product.payload OR $existing.dependencies != $product.dependencies OR $existing.producer != $product.producer OR $existing.interpretation != $product.interpretation { THROW 'immutable product identity reused'; };
        LET $root = SELECT * FROM ONLY type::record('canonical_roots', $product.key);
        IF $root = NONE { THROW 'product retention was explicitly released'; };
        IF $root.problem != $existing.problem OR $root.revision != $existing.revision OR $root.owner_kind != 'product' OR $root.owner != $existing.key { THROW 'product root disagrees with immutable origin'; };
    };
};
UPDATE type::record('canonical_stages', $operation) SET activated = true;
UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
UPSERT $version_guard SET key = 'version:' + $blob, generation = (generation ?? 0dec) + 1dec;
UPSERT type::record('canonical_guards', 'retention:' + $problem) SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
RETURN IF array::len($equivalent) != 0 { $equivalent[0].key } ELSE { $product.key };
"#;

const EDIT: &str = r#"
BEGIN;
LET $retention_guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $retention_guard FOR UPDATE;
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE;
LET $head_guard = type::record('canonical_guards', 'head:' + $problem);
SELECT * FROM $head_guard FOR UPDATE;
LET $ack = SELECT * FROM ONLY type::record('canonical_revisions', $operation);
IF $ack != NONE { THROW 'operation identity already committed; settle immutable revision'; };
LET $stage = SELECT * FROM ONLY type::record('canonical_stages', $operation);
IF $stage = NONE OR $stage.problem != $problem OR ($stage.expected_head ?? NONE) != $expected OR $stage.request_digest != $request_digest OR $stage.generation != $generation OR $stage.closed = false OR $stage.abandoned OR $stage.activated OR $stage.expires_at <= time::micros() { THROW 'closed staging manifest fenced or unavailable'; };
LET $head = SELECT * FROM ONLY type::record('canonical_problems', $problem);
IF ($head.head ?? NONE) != $expected { THROW 'head compare-and-set conflict'; };
LET $changes = SELECT * FROM canonical_staged_edits WHERE stage = $operation ORDER BY ordinal LIMIT /* EDIT_SCAN_LIMIT */;
IF array::len($changes) != $stage.edit_count OR array::len($changes) > /* EDIT_LIMIT */ { THROW 'closed staging metadata coverage mismatch'; };
LET $sequence = ($head.sequence ?? 0dec) + 1dec;
// Close all displaced intervals before checking the final namespace. Every
// physical page uses exact changed logical IDs; no unchanged source is rewritten.
FOR $batch IN array::clump($changes, 64) {
    LET $logicals = $batch.logical;
    LET $old = SELECT * FROM canonical_memberships WITH INDEX membership_logical WHERE problem = $problem AND logical IN $logicals AND to_sequence = NONE;
    LET $old_guards = array::distinct(array::concat(
        $old.map(|$member| type::string(['pse.scope-guard.v1', $problem, $member.scope])),
        $old.map(|$member| type::string(['pse.name-guard.v1', $problem, $member.scope, $member.name]))
    )).map(|$key| { id: type::record('canonical_guards', $key), key: $key, generation: 1dec });
    LET $existing = SELECT id FROM $old_guards.id;
    UPDATE $existing.id SET generation = generation + 1dec RETURN NONE;
    LET $existing_ids = $existing.id;
    LET $missing = $old_guards.filter(|$guard| $guard.id NOT IN $existing_ids);
    INSERT INTO canonical_guards $missing RETURN NONE;
    UPDATE $old.id SET to_sequence = $sequence RETURN NONE;
};
LET $scope_guards = array::distinct($changes.scope).map(|$scope| {
    LET $key = type::string(['pse.scope-guard.v1', $problem, $scope]);
    { id: type::record('canonical_guards', $key), key: $key, generation: 1dec }
});
FOR $guards IN array::clump($scope_guards, 64) {
    LET $existing = SELECT id FROM $guards.id;
    UPDATE $existing.id SET generation = generation + 1dec RETURN NONE;
    LET $existing_ids = $existing.id;
    LET $missing = $guards.filter(|$guard| $guard.id NOT IN $existing_ids);
    INSERT INTO canonical_guards $missing RETURN NONE;
};
LET $incoming = $changes.filter(|$change| $change.version != NONE);
LET $names = $incoming.map(|$change| type::string([$change.scope, $change.name]));
IF array::len(array::distinct($names)) != array::len($names) { THROW 'name already exists within incoming revision'; };
FOR $batch IN array::clump($changes, 64) {
    LET $creates = $batch.filter(|$change| $change.version != NONE);
    // Read exact guard IDs, update existing records, and insert absent records.
    // Released 3.3 UPSERT is create-first even for existing targets; recovering
    // that duplicate rebuilds the growing RocksDB transaction write index.
    // Every guard is actually written, so a concurrent premise writer conflicts
    // even for a previously absent record. Head/stage/retention decision locks
    // above still fence the complete transaction and its fresh retries.
    LET $guard_keys = array::distinct(array::concat(
        $batch.map(|$change| type::string(['pse.name-guard.v1', $problem, $change.scope, $change.name])),
        $creates.map(|$change| 'version:' + $change.version)
    ));
    LET $guard_rows = $guard_keys.map(|$key| { id: type::record('canonical_guards', $key), key: $key, generation: 1dec });
    LET $existing = SELECT id FROM $guard_rows.id;
    UPDATE $existing.id SET generation = generation + 1dec RETURN NONE;
    LET $existing_ids = $existing.id;
    LET $missing = $guard_rows.filter(|$guard| $guard.id NOT IN $existing_ids);
    INSERT INTO canonical_guards $missing RETURN NONE;
    LET $manifest_records = $creates.map(|$change| type::record('canonical_version_manifests', $change.version));
    LET $manifests = SELECT * FROM $manifest_records;
    IF array::len($manifests) != array::len($creates) { THROW 'staged immutable source manifest coverage invalid'; };
    FOR $change IN $creates {
        LET $manifest = $manifests.find(|$header| $header.key = $change.version);
        IF $manifest = NONE OR $manifest.closed = false OR $manifest.logical != $change.logical OR $manifest.interpretation != $interpretation { THROW 'staged immutable source manifest invalid'; };
    };
    // One native name selection per demanded namespace/page, with scalar
    // bindings so released 3.3 selects the compound namespace index. Scientific
    // logical identity is checked against the exact incoming name afterwards.
    FOR $scope IN array::distinct($creates.scope) {
        LET $scope_changes = $creates.filter(|$change| $change.scope = $scope);
        LET $names = $scope_changes.name;
        LET $survivors = SELECT name, logical FROM canonical_memberships WITH INDEX membership_selection WHERE problem = $problem AND scope = $scope AND name IN $names AND to_sequence = NONE;
        FOR $survivor IN $survivors {
            LET $replacement = $scope_changes.find(|$change| $change.name = $survivor.name);
            IF $replacement = NONE OR $survivor.logical != $replacement.logical { THROW 'name already exists'; };
        };
    };
    LET $memberships = $creates.map(|$change| {
        LET $key = type::string(['pse.membership.v1', $operation, $change.logical]);
        { id: type::record('canonical_memberships', $key), key: $key,
          in: type::record('canonical_problems', $problem), out: type::record('canonical_version_manifests', $change.version),
          problem: $problem, scope: $change.scope, name: $change.name, logical: $change.logical,
          version: $change.version, from_sequence: $sequence }
    });
    INSERT RELATION INTO canonical_memberships $memberships RETURN NONE;
};
CREATE type::record('canonical_revisions', $operation) SET key = $operation, problem = $problem, sequence = $sequence, parent = $expected, operation = $operation, request = $request, interpretation = $interpretation;
CREATE type::record('canonical_roots', 'history:' + $operation) SET key = 'history:' + $operation, problem = $problem, revision = $operation, sequence = $sequence, owner_kind = 'history', owner = $operation;
UPDATE type::record('canonical_stages', $operation) SET activated = true;
UPSERT type::record('canonical_problems', $problem) SET key = $problem, head = $operation, sequence = $sequence;
UPSERT $head_guard SET key = 'head:' + $problem, generation = (generation ?? 0dec) + 1dec;
UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
UPSERT $retention_guard SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
COMMIT;
"#;

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_server_unit {
    use super::*;
    use surrealdb::types::Number;
    fn version(key: &str, logical: &str, payload: &[u8]) -> ObjectVersion {
        ObjectVersion {
            key: key.into(),
            logical: logical.into(),
            kind: "test".into(),
            payload: payload.to_vec().into(),
            interpretation: wire::INTERPRETATION.into(),
        }
    }
    #[tokio::test]
    async fn query_cancellation_cannot_establish_empty_success() {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let address = url::Url::parse(&options.endpoint)
            .unwrap()
            .socket_addrs(|| None)
            .unwrap()[0];
        let short = connect_client(address, &options, std::time::Duration::from_millis(50))
            .await
            .unwrap();
        let result = bounded_query(short.query("BEGIN; SLEEP 1s; CREATE canonical_guards:late SET key = 'late', generation = 1dec; COMMIT;")).await;
        assert!(
            result.is_err(),
            "cancellation cannot become a completed empty transaction"
        );
        let absent: Option<Object> = request(store.db.select(("canonical_guards", "late")))
            .await
            .unwrap();
        assert!(absent.is_none());
        let mut empty = bounded_query(
            store
                .db
                .query("SELECT * FROM canonical_guards WHERE key = 'late';"),
        )
        .await
        .unwrap();
        assert!(
            empty.take::<Vec<Object>>(0).unwrap().is_empty(),
            "a completed empty selector remains valid"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn large_activation_commits_3500_tiny_objects() {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let edits = (0..3500)
            .map(|ordinal| {
                let logical = format!("object-{ordinal}");
                ObjectEdit {
                    logical: logical.clone(),
                    scope: "root".into(),
                    name: logical.clone(),
                    version: Some(version(&format!("version-{ordinal}"), &logical, &[1])),
                    references: Vec::new(),
                }
            })
            .collect::<Vec<_>>();
        let result = store
            .edit(
                "large-activation",
                None,
                "large-activation-operation",
                &edits,
            )
            .await;
        let revision = result.expect("3500 separately staged objects must activate atomically");
        assert_eq!(revision.sequence, 1);
        // Duplicate incoming names straddling a physical activation page still
        // refuse the complete revision, with no partially visible memberships.
        let duplicate = (0..65)
            .map(|ordinal| {
                let logical = format!("duplicate-{ordinal}");
                ObjectEdit {
                    logical: logical.clone(),
                    scope: "root".into(),
                    name: format!("duplicate-name-{}", ordinal % 64),
                    version: Some(version(
                        &format!("duplicate-version-{ordinal}"),
                        &logical,
                        &[2],
                    )),
                    references: Vec::new(),
                }
            })
            .collect::<Vec<_>>();
        assert!(
            store
                .edit(
                    "large-activation",
                    Some(&revision.key),
                    "duplicate-names",
                    &duplicate
                )
                .await
                .is_err()
        );
        let head: Option<Object> =
            request(store.db.select(("canonical_problems", "large-activation")))
                .await
                .unwrap();
        let head = wire::decode_canonical_problems(head.unwrap()).unwrap();
        assert_eq!(head.head, revision.key);
        assert_eq!(head.sequence, revision.sequence);
        assert!(store.revision("duplicate-names").await.unwrap().is_none());
        assert_eq!(store.inventory(&revision).await.unwrap().len(), 3500);
        // All old intervals close before any replacements open: a swap of names
        // is legal, while the retained historical revision keeps its old meaning.
        let swap = [0, 3499].map(|ordinal| {
            let logical = format!("object-{ordinal}");
            ObjectEdit {
                logical: logical.clone(),
                scope: "root".into(),
                name: format!("object-{}", 3499 - ordinal),
                version: Some(version(&format!("version-{ordinal}"), &logical, &[1])),
                references: Vec::new(),
            }
        });
        let swapped = store
            .edit(
                "large-activation",
                Some(&revision.key),
                "swapped-names",
                &swap,
            )
            .await
            .unwrap();
        let old = store
            .protect(revision, std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let new = store
            .protect(swapped, std::time::Duration::from_secs(30))
            .await
            .unwrap();
        assert_eq!(
            store
                .select_names(&old, "root", &["object-0".into()])
                .await
                .unwrap()[0]
                .logical,
            "object-0"
        );
        assert_eq!(
            store
                .select_names(&new, "root", &["object-0".into()])
                .await
                .unwrap()[0]
                .logical,
            "object-3499"
        );
        store.release(&old).await.unwrap();
        store.release(&new).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn generated_native_scientific_cells_and_schema_identity() {
        use crate::canonical_codec as codec;
        use pse_schema::model::{
            Authority, ColumnRole, DerivationGranularity, FieldContract, Namespace, RelationDecl,
            SnapshotClass,
        };
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = CanonicalStore::connect(&options).await.unwrap();
        assert!(
            store.open().await.is_err(),
            "normal open must not install missing schema"
        );
        store.create().await.unwrap();
        let mut builder = pse_schema::RegistryBuilder::new();
        pse_schema::catalog::declare(&mut builder);
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Runtime,
                "canonical_wire_fixture",
                1,
                Authority::Derived,
                SnapshotClass::Sidecar,
                "Native exact codec fixture.",
            )
            .pk(&["key"])
            .granularity(DerivationGranularity::Row)
            .columns(vec![
                FieldContract::native(arrow_schema::DataType::Utf8)
                    .with_name("key")
                    .with_role(ColumnRole::Key),
                FieldContract::native(arrow_schema::DataType::Float64).with_name("finite"),
                FieldContract::native(arrow_schema::DataType::Float64)
                    .with_name("optional")
                    .optional(),
                FieldContract::native(arrow_schema::DataType::Binary).with_name("raw"),
            ]),
        );
        let registry = builder.build().unwrap();
        let schema = pse_codegen::codegen::canonical_schema(&registry).unwrap();
        let fixture = schema
            .lines()
            .filter(|line| {
                line.starts_with("DEFINE TABLE canonical_wire_fixture ")
                    || line.contains(" ON canonical_wire_fixture ")
            })
            .collect::<Vec<_>>()
            .join("\n");
        store.db.query(fixture).await.unwrap().check().unwrap();
        for (index, value) in [
            0.0f64,
            -0.0,
            f64::MIN_POSITIVE,
            f64::from_bits(1),
            f64::MAX,
            -17.25,
        ]
        .into_iter()
        .enumerate()
        {
            let mut object = Object::new();
            object.insert("key", format!("cell-{index}"));
            object.insert("finite", codec::encode_finite(value).unwrap());
            object.insert("raw", codec::encode_diagnostic_bits(0x7ff8_0000_0000_0042));
            let saved: Option<Object> = store
                .db
                .create(("canonical_wire_fixture", format!("cell-{index}")))
                .content(object)
                .await
                .unwrap();
            let mut saved = saved.unwrap();
            assert_eq!(
                codec::decode_finite(saved.remove("finite").unwrap())
                    .unwrap()
                    .to_bits(),
                value.to_bits()
            );
            assert!(
                saved
                    .get("optional")
                    .is_none_or(|value| matches!(value, Value::None))
            );
            assert_eq!(
                codec::decode_diagnostic_bits(saved.remove("raw").unwrap()).unwrap(),
                0x7ff8_0000_0000_0042
            );
        }
        // Deliberately malformed native input cannot become a valid scientific cell.
        let mut short = codec::encode_finite(1.0).unwrap();
        if let Value::Object(ref mut object) = short {
            object.insert("bits", surrealdb::types::Bytes::from(vec![0u8; 7]));
        }
        assert!(
            store
                .db
                .query(
                    "CREATE canonical_wire_fixture:bad SET key = 'bad', finite = $cell, raw = $raw;"
                )
                .bind(("cell", short))
                .bind(("raw", codec::encode_diagnostic_bits(0)))
                .await
                .unwrap()
                .check()
                .is_err()
        );
        for absent in [Value::None, Value::Null] {
            assert!(store.db.query("CREATE canonical_wire_fixture:absent SET key = 'absent', finite = $cell, raw = $raw;").bind(("cell", absent)).bind(("raw", codec::encode_diagnostic_bits(0))).await.unwrap().check().is_err());
        }
        let mut corrupt = codec::encode_finite(2.0).unwrap();
        if let Value::Object(ref mut object) = corrupt {
            object.insert("projection", 3.0);
        }
        let mut result = store
            .db
            .query("RETURN $cell;")
            .bind(("cell", corrupt))
            .await
            .unwrap();
        assert!(codec::decode_finite(result.take::<Value>(0).unwrap()).is_err());
        assert!(
            codec::check_record_identity(
                codec::record_link("other_table", "x"),
                "canonical_wire_fixture",
                "x"
            )
            .is_err()
        );
        store
            .db
            .query("UPDATE canonical_interpretations:current SET schema_digest = 'unknown-reader';")
            .await
            .unwrap()
            .check()
            .unwrap();
        assert!(matches!(
            store.open().await,
            Err(CanonicalError::Interpretation { .. })
        ));
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn grpc_exact_cells_and_guarded_revisions() {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        store.open().await.unwrap();
        let result: Result<(), CanonicalError> = async {
            for (index, value) in [0, (1u64<<63)-1, 1u64<<63, (1u64<<63)+1, u64::MAX-1, u64::MAX].into_iter().enumerate() {
                let row = pse_model::generated::runtime::canonical_guards::Row { key: format!("boundary-{index}"), generation: value };
                let saved: Option<Object> = store.db.create(("canonical_guards", row.key.as_str())).content(wire::encode_canonical_guards(&row)?).await?;
                assert_eq!(wire::decode_canonical_guards(saved.unwrap())?.generation, value);
            }
            let mut ordered = store.db.query("SELECT generation FROM canonical_guards WHERE generation >= $lower ORDER BY generation;").bind(("lower", crate::canonical_codec::encode_uint(1u64<<63)?)).await.and_then(checked)?;
            let rows: Vec<Object> = ordered.take(0)?;
            let ordered = rows.into_iter().map(|mut row| crate::canonical_codec::decode_uint(row.remove("generation").unwrap())).collect::<Result<Vec<_>,_>>()?;
            assert_eq!(ordered, [1u64<<63, (1u64<<63)+1, u64::MAX-1, u64::MAX]);
            for value in ["-1", "0.5", "18446744073709551616"] {
                let response = store.db.query("CREATE canonical_guards:invalid SET key = 'invalid', generation = $generation;").bind(("generation", Value::Number(Number::Decimal(value.parse().unwrap())))).await?;
                assert!(response.check().is_err());
            }
            let initial = vec![ObjectEdit { logical: "x".into(), scope: "root".into(), name: "x".into(), version: Some(version("x-v1", "x", &[1,2,3])), references: vec![] }, ObjectEdit { logical: "y".into(), scope: "root".into(), name: "y".into(), version: Some(version("y-v1", "y", &[4,5])), references: vec![] }];
            let first = store.edit("problem", None, "op-1", &initial).await.map_err(|error| CanonicalError::Configuration(format!("initial edit: {error}")))?;
            assert_eq!(first.sequence, 1);
            assert_eq!(store.edit("problem", None, "op-1", &initial).await?, first);
            assert!(store.edit("problem", None, "op-1", &[]).await.is_err());
            let changed = [ObjectEdit { logical: "x".into(), scope: "root".into(), name: "x".into(), version: Some(version("x-v2", "x", &[9])), references: vec![] }];
            let second = store.edit("problem", Some("op-1"), "op-2", &changed).await.map_err(|error| CanonicalError::Configuration(format!("second edit: {error}")))?;
            assert_eq!(second.sequence, 2);
            assert!(store.edit("problem", Some("op-1"), "op-stale", &[]).await.is_err());
            let old = store.inventory(&first).await.map_err(|error| CanonicalError::Configuration(format!("old inventory: {error}")))?;
            let new = store.inventory(&second).await.map_err(|error| CanonicalError::Configuration(format!("new inventory: {error}")))?;
            assert_eq!(old.iter().find(|member| member.logical == "x").unwrap().version, "x-v1");
            assert_eq!(new.iter().find(|member| member.logical == "x").unwrap().version, "x-v2");
            assert_eq!(new.iter().find(|member| member.logical == "y").unwrap().version, "y-v1");
            assert_eq!(store.object("x-v1").await?.unwrap().payload.as_slice(), [1,2,3]);
            let selection = store.protect(first, std::time::Duration::from_secs(30)).await.map_err(|error| CanonicalError::Configuration(format!("protection: {error}")))?;
            let selected = store.select_names(&selection, "root", &["x".into()]).await.map_err(|error| CanonicalError::Configuration(format!("selected names: {error}")))?;
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0].version, "x-v1");
            store.release(&selection).await?;
            assert!(store.select_names(&selection, "root", &["x".into()]).await.is_err());
            Ok(())
        }.await;
        store
            .db
            .query("REMOVE DATABASE $database;")
            .bind(("database", options.database))
            .await
            .unwrap()
            .check()
            .unwrap();
        result.unwrap();
    }
}
